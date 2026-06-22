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
  (say "usage: tools/send-lisp.scm [--device DEVICE] FORM")
  (say "")
  (say "Sends one Lisp form to the firmware console."))

(define (parse args device forms)
  (cond
    ((null? args) (values device (reverse forms)))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--device")
     (if (null? (cdr args))
         (die "--device needs a path")
         (parse (cddr args) (cadr args) forms)))
    (else
     (parse (cdr args) device (cons (car args) forms)))))

(define-values (device parts)
  (parse (command-line-tail)
         (or (env "PSOC_SERIAL") "/dev/ttyACM0")
         '()))

(when (null? parts)
  (usage)
  (exit 1))

(define (join-form parts)
  (let loop ((items parts) (out ""))
    (cond
      ((null? items) out)
      ((string=? out "") (loop (cdr items) (car items)))
      (else (loop (cdr items) (string-append out " " (car items)))))))

(run (string-append
      "printf '%s\\r' "
      (shell-quote (join-form parts))
      " > "
      (shell-quote device)))
