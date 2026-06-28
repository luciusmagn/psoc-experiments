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

(define default-credentials ".local/wifi/selected.env")
(define default-output-dir ".local/wifi/credentials")

(define (usage)
  (say "usage: tools/prepare-wifi-credential-blobs.scm [--check] [--credentials FILE] [--output-dir DIR]")
  (say "")
  (say "Writes ignored SSID/passphrase byte blobs for wifi-local-credentials builds.")
  (say "Credential values and SSIDs are never printed."))

(define (parse args credentials output-dir check-only)
  (cond
    ((null? args) (values credentials output-dir check-only))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--check")
     (parse (cdr args) credentials output-dir #t))
    ((string=? (car args) "--credentials")
     (if (null? (cdr args))
         (die "--credentials needs a path")
         (parse (cddr args) (cadr args) output-dir check-only)))
    ((string=? (car args) "--output-dir")
     (if (null? (cdr args))
         (die "--output-dir needs a path")
         (parse (cddr args) credentials (cadr args) check-only)))
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

(define (substring? text start wanted)
  (let ((wanted-len (string-length wanted))
        (text-len (string-length text)))
    (and (<= (+ start wanted-len) text-len)
         (string=? wanted (substring text start (+ start wanted-len))))))

(define (shell-single-unquote text)
  (let ((len (string-length text)))
    (when (or (= len 0) (not (char=? (string-ref text 0) #\')))
      (die "credential env value is not single-quoted"))
    (let ((port (open-output-string)))
      (let loop ((index 1))
        (when (>= index len)
          (die "unterminated single-quoted credential env value"))
        (let ((char (string-ref text index)))
          (cond
            ((char=? char #\')
             (cond
               ((= (+ index 1) len)
                (get-output-string port))
               ((substring? text (+ index 1) "\\''")
                (put-char port #\')
                (loop (+ index 4)))
               (else
                (die "unsupported shell quoting in credential env value"))))
            (else
             (put-char port char)
             (loop (+ index 1)))))))))

(define (line-binding line)
  (let loop ((index 0))
    (cond
      ((>= index (string-length line)) (values #f #f))
      ((char=? (string-ref line index) #\=)
       (values (substring line 0 index)
               (shell-single-unquote
                (substring line (+ index 1) (string-length line)))))
      (else (loop (+ index 1))))))

(define (env-file-value lines name)
  (let loop ((items lines))
    (cond
      ((null? items) #f)
      (else
       (let-values (((key value) (line-binding (car items))))
         (if (and key (string=? key name))
             value
             (loop (cdr items))))))))

(define (byte-string? text)
  (let loop ((index 0))
    (cond
      ((>= index (string-length text)) #t)
      ((<= (char->integer (string-ref text index)) 255) (loop (+ index 1)))
      (else #f))))

(define (validate-field name text min-len max-len)
  (unless text
    (die (string-append "credentials file is missing " name)))
  (unless (byte-string? text)
    (die (string-append name " contains a non-byte character")))
  (let ((len (string-length text)))
    (when (or (< len min-len) (> len max-len))
      (die (string-append name " length is outside the supported range")))
    len))

(define (write-byte-file path text)
  (run (string-append "mkdir -p " (shell-quote (dirname path))))
  (run (string-append "chmod 700 " (shell-quote (dirname path))))
  (let ((port (open-file-output-port
               path
               (file-options replace)
               (buffer-mode block)
               #f)))
    (let loop ((index 0))
      (when (< index (string-length text))
        (put-u8 port (char->integer (string-ref text index)))
        (loop (+ index 1))))
    (close-output-port port))
  (run (string-append "chmod 600 " (shell-quote path))))

(define (file-size path)
  (let ((text (capture-first-line
               (string-append "stat -c%s " (shell-quote path)))))
    (and (not (string=? text "")) (string->number text))))

(define (check-byte-file path expected-size)
  (unless (file-exists? path)
    (die (string-append "missing output " path)))
  (unless (= (file-size path) expected-size)
    (die (string-append "output size mismatch " path))))

(define-values (credentials output-dir check-only)
  (parse (command-line-tail)
         default-credentials
         default-output-dir
         #f))

(define credentials-path (repo-path credentials))
(unless (file-exists? credentials-path)
  (die (string-append "missing credentials file " credentials)))

(define credential-lines (read-lines credentials-path))
(define ssid (env-file-value credential-lines "WIFI_SSID"))
(define passphrase (env-file-value credential-lines "WIFI_PASSPHRASE"))
(define ssid-len (validate-field "WIFI_SSID" ssid 1 32))
(define passphrase-len (validate-field "WIFI_PASSPHRASE" passphrase 8 63))
(define output-path (repo-path output-dir))
(define ssid-path (path-join output-path "ssid.bin"))
(define passphrase-path (path-join output-path "passphrase.bin"))

(if check-only
    (begin
      (check-byte-file ssid-path ssid-len)
      (check-byte-file passphrase-path passphrase-len))
    (begin
      (write-byte-file ssid-path ssid)
      (write-byte-file passphrase-path passphrase)))

(say (string-append "credentials=" credentials))
(say (string-append "output-dir=" output-dir))
(say (string-append "ssid.bytes=" (number->string ssid-len)))
(say (string-append "passphrase.bytes=" (number->string passphrase-len)))
