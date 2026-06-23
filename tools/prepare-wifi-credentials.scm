#!/usr/bin/env -S scheme --script

(load
 (let* ((script (car (command-line)))
        (len (string-length script)))
   (let loop ((index (- len 1)))
     (cond
       ((< index 0) "psoc-common.scm")
       ((char=? (string-ref script index) #\/)
        (string-append (substring script 0 index) "/psoc-common.scm"))
       (else (loop (- index 1)))))))

(define default-iwd-dir "/var/lib/iwd")
(define default-output ".local/wifi/selected.env")
(define default-output-dir ".local/wifi/profiles")

(define (usage)
  (say "usage: tools/prepare-wifi-credentials.scm [--check] [--all] [--ssid SSID] [--profile FILE] [--iwd-dir DIR] [--output FILE] [--output-dir DIR]")
  (say "")
  (say "Writes non-enterprise IWD PSK profiles to ignored local state.")
  (say "Selected output defaults to .local/wifi/selected.env.")
  (say "All-profile output defaults to .local/wifi/profiles/.")
  (say "Credential values and SSIDs are never printed."))

(define (parse args ssid profile iwd-dir output output-dir check-only all-profiles)
  (cond
    ((null? args) (values ssid profile iwd-dir output output-dir check-only all-profiles))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--check")
     (parse (cdr args) ssid profile iwd-dir output output-dir #t all-profiles))
    ((string=? (car args) "--all")
     (parse (cdr args) ssid profile iwd-dir output output-dir check-only #t))
    ((string=? (car args) "--ssid")
     (if (null? (cdr args))
         (die "--ssid needs a value")
         (parse (cddr args) (cadr args) profile iwd-dir output output-dir check-only all-profiles)))
    ((string=? (car args) "--profile")
     (if (null? (cdr args))
         (die "--profile needs a file name")
         (parse (cddr args) ssid (cadr args) iwd-dir output output-dir check-only all-profiles)))
    ((string=? (car args) "--iwd-dir")
     (if (null? (cdr args))
         (die "--iwd-dir needs a directory")
         (parse (cddr args) ssid profile (cadr args) output output-dir check-only all-profiles)))
    ((string=? (car args) "--output")
     (if (null? (cdr args))
         (die "--output needs a path")
         (parse (cddr args) ssid profile iwd-dir (cadr args) output-dir check-only all-profiles)))
    ((string=? (car args) "--output-dir")
     (if (null? (cdr args))
         (die "--output-dir needs a directory")
         (parse (cddr args) ssid profile iwd-dir output (cadr args) check-only all-profiles)))
    (else
     (die (string-append "unknown argument " (car args))))))

(define (read-lines path)
  (call-with-input-file path
    (lambda (port)
      (let loop ((lines '()))
        (let ((line (get-line port)))
          (if (eof-object? line)
              (reverse lines)
              (loop (cons line lines))))))))

(define (capture-lines command capture-path)
  (ensure-local-dir)
  (let ((status (system (string-append command " > " (shell-quote capture-path)))))
    (unless (zero? status)
      (die (string-append "capture command failed with status "
                          (number->string status)))))
  (if (file-exists? capture-path)
      (read-lines capture-path)
      '()))

(define (string-suffix? suffix text)
  (let ((suffix-len (string-length suffix))
        (text-len (string-length text)))
    (and (<= suffix-len text-len)
         (string=? suffix (substring text (- text-len suffix-len) text-len)))))

(define (strip-suffix suffix text)
  (if (string-suffix? suffix text)
      (substring text 0 (- (string-length text) (string-length suffix)))
      text))

(define (hex-digit? char)
  (or (and (char>=? char #\0) (char<=? char #\9))
      (and (char>=? char #\a) (char<=? char #\f))
      (and (char>=? char #\A) (char<=? char #\F))))

(define (hex-value char)
  (cond
    ((and (char>=? char #\0) (char<=? char #\9))
     (- (char->integer char) (char->integer #\0)))
    ((and (char>=? char #\a) (char<=? char #\f))
     (+ 10 (- (char->integer char) (char->integer #\a))))
    (else
     (+ 10 (- (char->integer char) (char->integer #\A))))))

(define (hex-string? text)
  (let ((len (string-length text)))
    (and (even? len)
         (let loop ((index 0))
           (cond
             ((>= index len) #t)
             ((hex-digit? (string-ref text index)) (loop (+ index 1)))
             (else #f))))))

(define (decode-hex-name text)
  (let ((port (open-output-string)))
    (let loop ((index 0))
      (when (< index (string-length text))
        (let* ((high (hex-value (string-ref text index)))
               (low (hex-value (string-ref text (+ index 1))))
               (byte (+ (* high 16) low)))
          (put-char port (integer->char byte)))
        (loop (+ index 2))))
    (get-output-string port)))

(define (profile-ssid file-name)
  (let ((stem (strip-suffix ".psk" file-name)))
    (if (and (> (string-length stem) 1)
             (char=? (string-ref stem 0) #\=)
             (hex-string? (substring stem 1 (string-length stem))))
        (decode-hex-name (substring stem 1 (string-length stem)))
        stem)))

(define (enterprise-profile? file-name)
  (or (string-suffix? ".8021x" file-name)
      (string-prefix? "eduroam" file-name)))

(define (key-line-value prefix line)
  (and (string-prefix? prefix line)
       (substring line (string-length prefix) (string-length line))))

(define (read-profile-secret path)
  (let loop ((lines (read-lines path)) (passphrase #f) (psk #f))
    (cond
      ((null? lines)
       (cond
         (passphrase (values "passphrase" passphrase))
         (psk (values "psk" psk))
         (else (values #f #f))))
      (else
       (let* ((line (car lines))
              (next-passphrase (or passphrase (key-line-value "Passphrase=" line)))
              (next-psk (or psk (key-line-value "PreSharedKey=" line))))
         (loop (cdr lines) next-passphrase next-psk))))))

(define-record-type candidate
  (fields file-name ssid secret-kind secret))

(define (candidate-from-file iwd-dir file-name)
  (if (or (enterprise-profile? file-name)
          (not (string-suffix? ".psk" file-name)))
      #f
      (let ((path (path-join iwd-dir file-name)))
        (let-values (((kind secret) (read-profile-secret path)))
          (and kind
               (make-candidate file-name
                               (profile-ssid file-name)
                               kind
                               secret))))))

(define (find-profiles iwd-dir)
  (let* ((capture-path (repo-path ".local/wifi/iwd-profiles.txt"))
         (command (string-append
                   "mkdir -p " (shell-quote (dirname capture-path))
                   " && find " (shell-quote iwd-dir)
                   " -maxdepth 1 -type f -name '*.psk' -printf '%f\\n' 2>/dev/null | LC_ALL=C sort"))
         (names (capture-lines command capture-path)))
    (let loop ((items names) (candidates '()))
      (cond
        ((null? items) (reverse candidates))
        (else
         (let ((candidate (candidate-from-file iwd-dir (car items))))
           (loop (cdr items)
                 (if candidate
                     (cons candidate candidates)
                     candidates))))))))

(define (current-ssid)
  (let* ((capture-path (repo-path ".local/wifi/current-ssid.txt"))
         (command "iw dev 2>/dev/null | awk '/^[[:space:]]*ssid / {sub(/^[[:space:]]*ssid /, \"\"); print; exit}'")
         (lines (capture-lines command capture-path)))
    (and (pair? lines)
         (not (string=? (car lines) ""))
         (car lines))))

(define (select-candidate candidates wanted-ssid wanted-profile)
  (cond
    (wanted-profile
     (let loop ((items candidates))
       (cond
         ((null? items) #f)
         ((string=? (candidate-file-name (car items)) wanted-profile) (car items))
         (else (loop (cdr items))))))
    (wanted-ssid
     (let loop ((items candidates))
       (cond
         ((null? items) #f)
         ((string=? (candidate-ssid (car items)) wanted-ssid) (car items))
         (else (loop (cdr items))))))
    (else
     (let ((connected (current-ssid)))
       (if connected
           (select-candidate candidates connected #f)
           (and (pair? candidates) (car candidates)))))))

(define (write-env output-path candidate)
  (let ((full-path (repo-path output-path)))
    (run (string-append "mkdir -p " (shell-quote (dirname full-path))))
    (when (file-exists? full-path)
      (delete-file full-path))
    (call-with-output-file full-path
      (lambda (port)
        (display "WIFI_SSID=" port)
        (display (shell-quote (candidate-ssid candidate)) port)
        (newline port)
        (display "WIFI_SECRET_KIND=" port)
        (display (shell-quote (candidate-secret-kind candidate)) port)
        (newline port)
        (if (string=? (candidate-secret-kind candidate) "passphrase")
            (display "WIFI_PASSPHRASE=" port)
            (display "WIFI_PSK=" port))
        (display (shell-quote (candidate-secret candidate)) port)
        (newline port)))
    (run (string-append "chmod 600 " (shell-quote full-path)))
    full-path))

(define (candidate-output-path output-dir index)
  (string-append output-dir
                 "/profile-"
                 (number->string index)
                 ".env"))

(define (write-all-env output-dir candidates)
  (let ((full-dir (repo-path output-dir)))
    (run (string-append "mkdir -p " (shell-quote full-dir)))
    (run (string-append "chmod 700 " (shell-quote full-dir)))
    (run (string-append "find " (shell-quote full-dir)
                        " -maxdepth 1 -type f -name 'profile-*.env' -delete"))
    (let loop ((items candidates) (index 1))
      (unless (null? items)
        (write-env (candidate-output-path output-dir index) (car items))
        (loop (cdr items) (+ index 1)))))
  (length candidates))

(define (env-value-length path name)
  (let ((prefix (string-append name "=")))
    (let loop ((lines (if (file-exists? path) (read-lines path) '())))
      (cond
        ((null? lines) #f)
        ((string-prefix? prefix (car lines))
         (string-length (substring (car lines)
                                   (string-length prefix)
                                   (string-length (car lines)))))
        (else (loop (cdr lines)))))))

(define (report-output output-path)
  (let* ((full-path (repo-path output-path))
         (ssid-len (env-value-length full-path "WIFI_SSID"))
         (pass-len (env-value-length full-path "WIFI_PASSPHRASE"))
         (psk-len (env-value-length full-path "WIFI_PSK"))
         (kind-len (env-value-length full-path "WIFI_SECRET_KIND")))
    (if ssid-len
        (begin
          (say (string-append "output=" output-path))
          (say (string-append "ssid-field-bytes=" (number->string ssid-len)))
          (say (string-append "secret-kind-field-bytes="
                              (number->string (or kind-len 0))))
          (say (string-append "secret-field-bytes="
                              (number->string (or pass-len psk-len 0)))))
        (die (string-append "missing output " output-path)))))

(define (count-profile-env-files output-dir)
  (let* ((full-dir (repo-path output-dir))
         (capture-path (repo-path ".local/wifi/profile-env-files.txt"))
         (command (string-append
                   "mkdir -p " (shell-quote (dirname capture-path))
                   " && find " (shell-quote full-dir)
                   " -maxdepth 1 -type f -name 'profile-*.env' -printf '%f\\n' 2>/dev/null | LC_ALL=C sort"))
         (names (capture-lines command capture-path)))
    (length names)))

(define (report-all-output output-dir)
  (say (string-append "output-dir=" output-dir))
  (say (string-append "profile-env-count="
                      (number->string (count-profile-env-files output-dir)))))

(define-values (wanted-ssid wanted-profile iwd-dir output-path output-dir check-only all-profiles)
  (parse (command-line-tail)
         (env "WIFI_SSID")
         (env "WIFI_PROFILE")
         (or (env "IWD_DIR") default-iwd-dir)
         default-output
         default-output-dir
         #f
         #f))

(if check-only
    (begin
      (report-output output-path)
      (when all-profiles
        (report-all-output output-dir)))
    (let* ((candidates (find-profiles iwd-dir))
           (selected (select-candidate candidates wanted-ssid wanted-profile)))
      (unless selected
        (die "no matching non-enterprise IWD PSK profile with a stored secret"))
      (write-env output-path selected)
      (when all-profiles
        (write-all-env output-dir candidates))
      (say (string-append "selected-profile-count="
                          (number->string (length candidates))))
      (when all-profiles
        (report-all-output output-dir))
      (report-output output-path)))
