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
(define default-character-delay-ms 500)
(define open-settle-delay-ms 1000)
(define default-restore-delay-ms 25000)
(define default-hold-before-join-cr-ms 0)

(load-shared-object "libc.so.6")
(define usleep (foreign-procedure "usleep" (unsigned-int) int))

(define (usage)
  (say "usage: tools/send-wifi-join.scm [--check] [--device DEVICE] [--credentials FILE] [--delay-ms MS] [--restore-delay-ms MS] [--hold-before-join-cr-ms MS]")
  (say "")
  (say "Sends selected local Wi-Fi credentials to (wifi-join-wpa2 ...) with console echo disabled.")
  (say "Credential values and SSIDs are not printed."))

(define (parse-ms name text)
  (let ((value (string->number text)))
    (if (and value (integer? value) (exact? value) (>= value 0))
        value
        (die (string-append name " needs a non-negative integer")))))

(define (parse args device credentials delay-ms restore-delay-ms hold-before-join-cr-ms check-only)
  (cond
    ((null? args) (values device credentials delay-ms restore-delay-ms hold-before-join-cr-ms check-only))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--check")
     (parse (cdr args) device credentials delay-ms restore-delay-ms hold-before-join-cr-ms #t))
    ((string=? (car args) "--device")
     (if (null? (cdr args))
         (die "--device needs a path")
         (parse (cddr args) (cadr args) credentials delay-ms restore-delay-ms hold-before-join-cr-ms check-only)))
    ((string=? (car args) "--credentials")
     (if (null? (cdr args))
         (die "--credentials needs a path")
         (parse (cddr args) device (cadr args) delay-ms restore-delay-ms hold-before-join-cr-ms check-only)))
    ((string=? (car args) "--delay-ms")
     (if (null? (cdr args))
         (die "--delay-ms needs a value")
         (parse (cddr args) device credentials (parse-ms "--delay-ms" (cadr args)) restore-delay-ms hold-before-join-cr-ms check-only)))
    ((string=? (car args) "--restore-delay-ms")
     (if (null? (cdr args))
         (die "--restore-delay-ms needs a value")
         (parse (cddr args) device credentials delay-ms (parse-ms "--restore-delay-ms" (cadr args)) hold-before-join-cr-ms check-only)))
    ((string=? (car args) "--hold-before-join-cr-ms")
     (if (null? (cdr args))
         (die "--hold-before-join-cr-ms needs a value")
         (parse (cddr args) device credentials delay-ms restore-delay-ms (parse-ms "--hold-before-join-cr-ms" (cadr args)) check-only)))
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

(define (sleep-ms ms)
  (when (> ms 0)
    (usleep (* ms 1000))))

(define (max-left left right)
  (if (> left right) left right))

(define (put-char-byte port char)
  (let ((byte (char->integer char)))
    (when (> byte 255)
      (die "form contains a non-byte character"))
    (put-u8 port byte)))

(define (send-paced-text device form delay-ms final-cr?)
  (say (string-append
        "+ send "
        (number->string (string-length form))
        " bytes to "
        device
        " with "
        (number->string delay-ms)
        " ms pacing"))
  (let ((port (open-file-output-port
               device
               (file-options no-fail)
               (buffer-mode none))))
    (sleep-ms (max-left open-settle-delay-ms delay-ms))
    (let loop ((index 0))
      (when (< index (string-length form))
        (put-char-byte port (string-ref form index))
        (flush-output-port port)
        (sleep-ms delay-ms)
        (loop (+ index 1))))
    (when final-cr?
      (put-u8 port 13))
    (flush-output-port port)
    (close-output-port port)))

(define (send-paced-form device form delay-ms)
  (send-paced-text device form delay-ms #t))

(define (send-carriage-return device)
  (say (string-append "+ send carriage return to " device))
  (let ((port (open-file-output-port
               device
               (file-options no-fail)
               (buffer-mode none))))
    (put-u8 port 13)
    (flush-output-port port)
    (close-output-port port)))

(define (lisp-string text)
  (let ((port (open-output-string)))
    (put-char port #\")
    (let loop ((index 0))
      (when (< index (string-length text))
        (let* ((char (string-ref text index))
               (byte (char->integer char)))
          (cond
            ((char=? char #\") (display "\\\"" port))
            ((char=? char #\\) (display "\\\\" port))
            ((char=? char #\newline) (display "\\n" port))
            ((char=? char #\return) (display "\\r" port))
            ((char=? char #\tab) (display "\\t" port))
            ((and (>= byte 32) (<= byte 126)) (put-char port char))
            (else (die "credential contains an unsupported non-printable character"))))
        (loop (+ index 1))))
    (put-char port #\")
    (get-output-string port)))

(define-values (device credentials delay-ms restore-delay-ms hold-before-join-cr-ms check-only)
  (parse (command-line-tail)
         (or (env "PSOC_SERIAL") "/dev/ttyACM0")
         default-credentials
         default-character-delay-ms
         default-restore-delay-ms
         default-hold-before-join-cr-ms
         #f))

(define credentials-path (repo-path credentials))
(unless (file-exists? credentials-path)
  (die (string-append "missing credentials file " credentials)))

(define credential-lines (read-lines credentials-path))
(define ssid (env-file-value credential-lines "WIFI_SSID"))
(define passphrase (env-file-value credential-lines "WIFI_PASSPHRASE"))

(unless ssid
  (die "credentials file is missing WIFI_SSID"))
(unless passphrase
  (die "credentials file does not contain WIFI_PASSPHRASE; raw PSK join is not implemented"))

(say (string-append "credentials=" credentials))
(say (string-append "ssid.length=" (number->string (string-length ssid))))
(say (string-append "passphrase.length=" (number->string (string-length passphrase))))
(say (string-append "restore-delay-ms=" (number->string restore-delay-ms)))
(say (string-append "hold-before-join-cr-ms=" (number->string hold-before-join-cr-ms)))

(when check-only
  (exit 0))

(configure-serial-device device)

(define join-form
  (string-append "(wifi-join-wpa2 " (lisp-string ssid) " " (lisp-string passphrase) ")"))

(send-paced-form device "(console-echo off)" delay-ms)
(if (> hold-before-join-cr-ms 0)
    (begin
      (send-paced-text device join-form delay-ms #f)
      (say (string-append "+ hold before join carriage return "
                          (number->string hold-before-join-cr-ms)
                          " ms"))
      (sleep-ms hold-before-join-cr-ms)
      (send-carriage-return device))
    (send-paced-form device join-form delay-ms))
(sleep-ms restore-delay-ms)
(send-paced-form device "(console-echo on)" delay-ms)
