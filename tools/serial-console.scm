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
  (say "usage: tools/serial-console.scm [--log PATH] [--no-log] [--tail-log] [DEVICE]")
  (say "")
  (say "Opens the firmware UART console. Default device is PSOC_SERIAL or /dev/ttyACM0.")
  (say "By default, mirrors bytes to .local/logs/serial-console.log.")
  (say "--tail-log follows the log file without opening the UART."))

(define default-log-path ".local/logs/serial-console.log")

(define (absolute-path? path)
  (and (> (string-length path) 0)
       (char=? (string-ref path 0) #\/)))

(define (resolve-log-path path)
  (if (absolute-path? path)
      path
      (repo-path path)))

(define (prepare-log path)
  (run (string-append "mkdir -p " (shell-quote (dirname path))))
  (run (string-append "touch " (shell-quote path)))
  (run (string-append "chmod 600 " (shell-quote path))))

(define (parse args device log-path log? tail-log?)
  (cond
    ((null? args) (values device log-path log? tail-log?))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--log")
     (if (null? (cdr args))
         (die "--log needs a path")
         (parse (cddr args) device (cadr args) #t tail-log?)))
    ((string=? (car args) "--no-log")
     (parse (cdr args) device log-path #f tail-log?))
    ((string=? (car args) "--tail-log")
     (parse (cdr args) device log-path log? #t))
    ((string-prefix? "-" (car args))
     (die (string-append "unknown argument: " (car args))))
    (device
     (die "serial-console.scm accepts at most one device path"))
    (else
     (parse (cdr args) (car args) log-path log? tail-log?))))

(define-values (requested-device requested-log-path log? tail-log?)
  (parse (command-line-tail) #f default-log-path #t #f))

(define device
  (cond
    (requested-device requested-device)
    ((env "PSOC_SERIAL") (env "PSOC_SERIAL"))
    (else "/dev/ttyACM0")))

(define log-path (resolve-log-path requested-log-path))

(when tail-log?
  (prepare-log log-path)
  (run (string-append "tail -f " (shell-quote log-path)))
  (exit 0))

(configure-serial-device device)
(if log?
    (begin
      (prepare-log log-path)
      (run (string-append
            "cat "
            (shell-quote device)
            " | tee -a "
            (shell-quote log-path))))
    (run (string-append "cat " (shell-quote device))))
