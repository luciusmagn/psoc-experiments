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
  (say "usage: tools/serial-console.scm [DEVICE]")
  (say "")
  (say "Opens the firmware UART console. Default device is PSOC_SERIAL or /dev/ttyACM0."))

(define args (command-line-tail))
(when (and (pair? args) (string=? (car args) "--help"))
  (usage)
  (exit 0))
(when (> (length args) 1)
  (die "serial-console.scm accepts at most one device path"))

(define device
  (cond
    ((pair? args) (car args))
    ((env "PSOC_SERIAL") (env "PSOC_SERIAL"))
    (else "/dev/ttyACM0")))

(run (string-append
      "stty -F "
      (shell-quote device)
      " 115200 raw -echo -echoe -echok -echoctl -echoke -icanon min 1 time 0"))
(run (string-append "cat " (shell-quote device)))
