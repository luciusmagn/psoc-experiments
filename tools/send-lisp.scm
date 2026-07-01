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

(define (usage)
  (say
   "usage: tools/send-lisp.scm [--device DEVICE] [--delay-ms MS] [--post-cr-delay-ms MS] FORM")
  (say "")
  (say "Sends one Lisp form to the firmware console.")
  (say "Default pacing is 500 ms per byte after a 1s open-settle delay.")
  (say "Default post-CR hold is 500 ms before closing the UART writer.")
  (say "The form is not printed."))

(define default-character-delay-ms 500)
(define default-post-cr-delay-ms 500)
(define open-settle-delay-ms 1000)
(define burst-buffer ".local/serial/send-buffer.lisp")

(load-shared-object "libc.so.6")
(define usleep (foreign-procedure "usleep" (unsigned-int) int))

(define (parse-ms option text)
  (let ((value (string->number text)))
    (if (and value (integer? value) (exact? value) (>= value 0))
        value
        (die (string-append option " needs a non-negative integer")))))

(define (parse args device delay-ms post-cr-delay-ms forms)
  (cond
    ((null? args) (values device delay-ms post-cr-delay-ms (reverse forms)))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--device")
     (if (null? (cdr args))
         (die "--device needs a path")
         (parse (cddr args) (cadr args) delay-ms post-cr-delay-ms forms)))
    ((string=? (car args) "--delay-ms")
     (if (null? (cdr args))
         (die "--delay-ms needs a value")
         (parse (cddr args)
                device
                (parse-ms "--delay-ms" (cadr args))
                post-cr-delay-ms
                forms)))
    ((string=? (car args) "--post-cr-delay-ms")
     (if (null? (cdr args))
         (die "--post-cr-delay-ms needs a value")
         (parse (cddr args)
                device
                delay-ms
                (parse-ms "--post-cr-delay-ms" (cadr args))
                forms)))
    (else
     (parse (cdr args) device delay-ms post-cr-delay-ms (cons (car args) forms)))))

(define-values (device delay-ms post-cr-delay-ms parts)
  (parse (command-line-tail)
         (or (env "PSOC_SERIAL") "/dev/ttyACM0")
         default-character-delay-ms
         default-post-cr-delay-ms
         '()))

(when (null? parts)
  (usage)
  (exit 1))

(configure-serial-device device)

(define (join-form parts)
  (let loop ((items parts) (out ""))
    (cond
      ((null? items) out)
      ((string=? out "") (loop (cdr items) (car items)))
      (else (loop (cdr items) (string-append out " " (car items)))))))

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

(define (write-burst-buffer path form final-cr?)
  (run (string-append "mkdir -p " (shell-quote (dirname path))))
  (call-with-output-file path
    (lambda (port)
      (display form port)
      (when final-cr?
        (put-char port #\return)))
    'replace)
  (run (string-append "chmod 600 " (shell-quote path))))

(define (send-burst-text device form final-cr?)
  (let ((path (repo-path burst-buffer)))
    (write-burst-buffer path form final-cr?)
    (run (string-append "cat " (shell-quote path) " > " (shell-quote device)))
    (run (string-append "rm -f " (shell-quote path)))))

(define (send-paced-form device form delay-ms post-cr-delay-ms)
  (say (string-append
        "+ send "
        (number->string (string-length form))
        " bytes to "
        device
        " with "
        (number->string delay-ms)
        " ms pacing"))
  (sleep-ms (max-left open-settle-delay-ms delay-ms))
  (if (= delay-ms 0)
      (send-burst-text device form #t)
      (let ((port (open-file-output-port
                   device
                   (file-options no-fail)
                   (buffer-mode none))))
        (let loop ((index 0))
          (when (< index (string-length form))
            (put-char-byte port (string-ref form index))
            (flush-output-port port)
            (sleep-ms delay-ms)
            (loop (+ index 1))))
        (put-u8 port 13)
        (flush-output-port port)
        (sleep-ms post-cr-delay-ms)
        (close-output-port port))))

(send-paced-form device (join-form parts) delay-ms post-cr-delay-ms)
