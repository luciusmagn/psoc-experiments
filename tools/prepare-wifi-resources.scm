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

(define default-whd-url "https://github.com/Infineon/wifi-host-driver.git")
(define default-whd-tag "release-v3.2.0")
(define default-whd-dir ".local/refs/wifi-host-driver")
(define default-output-dir ".local/wifi/resources")

(define firmware-path
  "WiFi_Host_Driver/resources/firmware/COMPONENT_4343W/4343WA1.bin")
(define clm-path
  "WiFi_Host_Driver/resources/clm/COMPONENT_4343W/4343WA1.clm_blob")
(define nvram-header-path
  "WiFi_Host_Driver/resources/nvram/COMPONENT_4343W/COMPONENT_MURATA-1DX/wifi_nvram_image.h")
(define license-path
  "WiFi_Host_Driver/resources/LICENSE-permissive-binary-license-1.0.txt")

(define (usage)
  (say "usage: tools/prepare-wifi-resources.scm [--check] [--tag TAG] [--whd-dir DIR] [--output DIR] [--mac MAC]")
  (say "")
  (say "Fetches CYW4343W firmware, CLM, and board NVRAM resources into ignored local state.")
  (say "Output defaults to .local/wifi/resources. Binary resources are not committed."))

(define (parse args check? tag whd-dir output-dir mac)
  (cond
    ((null? args) (values check? tag whd-dir output-dir mac))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--check")
     (parse (cdr args) #t tag whd-dir output-dir mac))
    ((string=? (car args) "--tag")
     (if (null? (cdr args))
         (die "--tag needs a value")
         (parse (cddr args) check? (cadr args) whd-dir output-dir mac)))
    ((string=? (car args) "--whd-dir")
     (if (null? (cdr args))
         (die "--whd-dir needs a directory")
         (parse (cddr args) check? tag (cadr args) output-dir mac)))
    ((string=? (car args) "--output")
     (if (null? (cdr args))
         (die "--output needs a directory")
         (parse (cddr args) check? tag whd-dir (cadr args) mac)))
    ((string=? (car args) "--mac")
     (if (null? (cdr args))
         (die "--mac needs an address")
         (parse (cddr args) check? tag whd-dir output-dir (cadr args))))
    (else
     (die (string-append "unknown argument: " (car args))))))

(define (read-lines path)
  (call-with-input-file path
    (lambda (port)
      (let loop ((lines '()))
        (let ((line (get-line port)))
          (if (eof-object? line)
              (reverse lines)
              (loop (cons line lines))))))))

(define (write-string-file path text)
  (run (string-append "mkdir -p " (shell-quote (dirname path))))
  (call-with-output-file path
    (lambda (port)
      (display text port))
    'replace))

(define (copy-resource whd-dir tag source output)
  (run (string-append
        "git -C "
        (shell-quote whd-dir)
        " show "
        (shell-quote (string-append tag ":" source))
        " > "
        (shell-quote output))))

(define (ensure-whd-reference whd-dir)
  (if (file-exists? (path-join whd-dir ".git"))
      (run (string-append "git -C " (shell-quote whd-dir) " fetch --tags origin"))
      (begin
        (run (string-append "mkdir -p " (shell-quote (dirname whd-dir))))
        (run (string-append
              "git clone "
              (shell-quote default-whd-url)
              " "
              (shell-quote whd-dir))))))

(define (path-size path)
  (capture-first-line
   (string-append "stat -c%s " (shell-quote path))))

(define (path-sha256 path)
  (capture-first-line
   (string-append "sha256sum " (shell-quote path) " | awk '{print $1}'")))

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

(define (valid-mac? text)
  (and (= (string-length text) 17)
       (let loop ((index 0))
         (cond
           ((= index 17) #t)
           ((or (= index 2) (= index 5) (= index 8) (= index 11) (= index 14))
            (and (char=? (string-ref text index) #\:)
                 (loop (+ index 1))))
           ((hex-digit? (string-ref text index)) (loop (+ index 1)))
           (else #f)))))

(define (generated-mac)
  (let* ((hex (capture-first-line "od -An -N5 -tx1 /dev/urandom | tr -d ' \\n'"))
         (b1 (substring hex 0 2))
         (b2 (substring hex 2 4))
         (b3 (substring hex 4 6))
         (b4 (substring hex 6 8))
         (b5 (substring hex 8 10)))
    (string-append "02:" b1 ":" b2 ":" b3 ":" b4 ":" b5)))

(define (read-mac path requested)
  (cond
    (requested
     (unless (valid-mac? requested)
       (die "invalid --mac value; expected xx:xx:xx:xx:xx:xx"))
     requested)
    ((file-exists? path)
     (let ((lines (read-lines path)))
       (if (and (pair? lines) (valid-mac? (car lines)))
           (car lines)
           (die (string-append "invalid stored MAC in " path)))))
    (else
     (let ((mac (generated-mac)))
       (write-string-file path (string-append mac "\n"))
       (run (string-append "chmod 600 " (shell-quote path)))
       mac))))

(define (string-contains? needle text)
  (let ((needle-len (string-length needle))
        (text-len (string-length text)))
    (let loop ((index 0))
      (cond
        ((> (+ index needle-len) text-len) #f)
        ((string=? needle (substring text index (+ index needle-len))) index)
        (else (loop (+ index 1)))))))

(define (starts-with-at? text index needle)
  (let ((end (+ index (string-length needle))))
    (and (<= end (string-length text))
         (string=? needle (substring text index end)))))

(define (decode-c-string text start output)
  (let loop ((index (+ start 1)))
    (cond
      ((>= index (string-length text))
       (die "unterminated C string in NVRAM header"))
      ((char=? (string-ref text index) #\")
       (+ index 1))
      ((char=? (string-ref text index) #\\)
       (when (>= (+ index 1) (string-length text))
         (die "dangling escape in NVRAM header"))
       (let ((next (string-ref text (+ index 1))))
         (cond
           ((char=? next #\x)
            (unless (and (< (+ index 3) (string-length text))
                         (hex-digit? (string-ref text (+ index 2)))
                         (hex-digit? (string-ref text (+ index 3))))
              (die "invalid hex escape in NVRAM header"))
            (let* ((high (hex-value (string-ref text (+ index 2))))
                   (low (hex-value (string-ref text (+ index 3))))
                   (byte (+ (* high 16) low)))
              (put-char output (integer->char byte))
              (loop (+ index 4))))
           ((char=? next #\0)
            (put-char output (integer->char 0))
            (loop (+ index 2)))
           ((char=? next #\n)
            (put-char output #\newline)
            (loop (+ index 2)))
           ((char=? next #\r)
            (put-char output #\return)
            (loop (+ index 2)))
           ((char=? next #\t)
            (put-char output #\tab)
            (loop (+ index 2)))
           (else
            (put-char output next)
            (loop (+ index 2))))))
      (else
       (put-char output (string-ref text index))
       (loop (+ index 1))))))

(define (write-nvram-line line mac-entry output)
  (let ((macro "NVRAM_GENERATED_MAC_ADDRESS"))
    (let loop ((index 0))
      (when (< index (string-length line))
        (cond
          ((char=? (string-ref line index) #\")
           (loop (decode-c-string line index output)))
          ((starts-with-at? line index macro)
           (display mac-entry output)
           (loop (+ index (string-length macro))))
          (else
           (loop (+ index 1))))))))

(define (write-nvram-bin header-path output-path mac)
  (let ((lines (read-lines header-path))
        (mac-entry (string-append "macaddr=" mac)))
    (call-with-output-file output-path
      (lambda (output)
        (let loop ((items lines) (inside? #f) (found? #f))
          (cond
            ((null? items)
             (unless found?
               (die "missing wifi_nvram_image[] in NVRAM header"))
             (when inside?
               (die "unterminated wifi_nvram_image[] in NVRAM header"))
             #t)
            (else
             (let* ((line (car items))
                    (line-start? (string-contains? "static const char wifi_nvram_image[]" line))
                    (start? (or inside? line-start?))
                    (done? (and start? (string-contains? ";" line))))
               (when start?
                 (write-nvram-line line mac-entry output))
               (if done?
                   #t
                   (loop (cdr items) start? (or found? line-start?))))))))
      'replace)))

(define (write-printable-nvram nvram-bin output-path)
  (run (string-append
        "tr '\\000' '\\n' < "
        (shell-quote nvram-bin)
        " | sed '/^$/d' > "
        (shell-quote output-path))))

(define (write-sha256s output-dir files)
  (run (string-append
        "cd "
        (shell-quote output-dir)
        " && sha256sum "
        (let loop ((items files) (out ""))
          (cond
            ((null? items) out)
            ((string=? out "") (loop (cdr items) (shell-quote (car items))))
            (else (loop (cdr items)
                        (string-append out " " (shell-quote (car items)))))))
        " > SHA256SUMS")))

(define (require-output path)
  (unless (file-exists? path)
    (die (string-append "missing resource: " path))))

(define (show-resource name path)
  (say (string-append name
                      " bytes="
                      (path-size path)
                      " sha256="
                      (path-sha256 path))))

(define-values (check? tag whd-dir-arg output-dir-arg requested-mac)
  (parse (command-line-tail)
         #f
         default-whd-tag
         default-whd-dir
         default-output-dir
         #f))

(define whd-dir (repo-path whd-dir-arg))
(define output-dir (repo-path output-dir-arg))
(define firmware-output (path-join output-dir "4343WA1.bin"))
(define clm-output (path-join output-dir "4343WA1.clm_blob"))
(define nvram-header-output (path-join output-dir "wifi_nvram_image.h"))
(define nvram-bin-output (path-join output-dir "wifi_nvram.bin"))
(define nvram-text-output (path-join output-dir "wifi_nvram.txt"))
(define mac-output (path-join output-dir "generated_mac_address.txt"))
(define license-output (path-join output-dir "LICENSE-permissive-binary-license-1.0.txt"))
(define source-output (path-join output-dir "source.env"))

(if check?
    (begin
      (require-output firmware-output)
      (require-output clm-output)
      (require-output nvram-bin-output)
      (require-output license-output))
    (begin
      (ensure-whd-reference whd-dir)
      (run (string-append "mkdir -p " (shell-quote output-dir)))
      (let ((mac (read-mac mac-output requested-mac)))
        (copy-resource whd-dir tag firmware-path firmware-output)
        (copy-resource whd-dir tag clm-path clm-output)
        (copy-resource whd-dir tag nvram-header-path nvram-header-output)
        (copy-resource whd-dir tag license-path license-output)
        (write-nvram-bin nvram-header-output nvram-bin-output mac)
        (write-printable-nvram nvram-bin-output nvram-text-output)
        (write-string-file source-output
                           (string-append
                            "WHD_URL=" default-whd-url "\n"
                            "WHD_TAG=" tag "\n"
                            "FIRMWARE_PATH=" firmware-path "\n"
                            "CLM_PATH=" clm-path "\n"
                            "NVRAM_HEADER_PATH=" nvram-header-path "\n"))
        (write-sha256s output-dir
                       '("4343WA1.bin"
                         "4343WA1.clm_blob"
                         "wifi_nvram.bin"
                         "wifi_nvram.txt"
                         "wifi_nvram_image.h"
                         "LICENSE-permissive-binary-license-1.0.txt"
                         "source.env")))))

(show-resource "firmware" firmware-output)
(show-resource "clm" clm-output)
(show-resource "nvram" nvram-bin-output)
(say (string-append "resources=" output-dir))
