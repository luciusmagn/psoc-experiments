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
  (say "usage: tools/send-lisp.scm [--device DEVICE] [--delay-ms MS] FORM")
  (say "")
  (say "Sends one Lisp form to the firmware console.")
  (say "Default pacing is 200 ms per byte; the form is not printed."))

(define default-character-delay-ms 200)

(load-shared-object "libc.so.6")
(define usleep (foreign-procedure "usleep" (unsigned-int) int))

(define (parse-delay-ms text)
  (let ((value (string->number text)))
    (if (and value (integer? value) (exact? value) (>= value 0))
        value
        (die "--delay-ms needs a non-negative integer"))))

(define (parse args device delay-ms forms)
  (cond
    ((null? args) (values device delay-ms (reverse forms)))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--device")
     (if (null? (cdr args))
         (die "--device needs a path")
         (parse (cddr args) (cadr args) delay-ms forms)))
    ((string=? (car args) "--delay-ms")
     (if (null? (cdr args))
         (die "--delay-ms needs a value")
         (parse (cddr args) device (parse-delay-ms (cadr args)) forms)))
    (else
     (parse (cdr args) device delay-ms (cons (car args) forms)))))

(define-values (device delay-ms parts)
  (parse (command-line-tail)
         (or (env "PSOC_SERIAL") "/dev/ttyACM0")
         default-character-delay-ms
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

(define (put-char-byte port char)
  (let ((byte (char->integer char)))
    (when (> byte 255)
      (die "form contains a non-byte character"))
    (put-u8 port byte)))

(define (send-paced-form device form delay-ms)
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
    (let loop ((index 0))
      (when (< index (string-length form))
        (put-char-byte port (string-ref form index))
        (flush-output-port port)
        (sleep-ms delay-ms)
        (loop (+ index 1))))
    (put-u8 port 13)
    (flush-output-port port)
    (close-output-port port)))

(send-paced-form device (join-form parts) delay-ms)
