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
  (say "usage: tools/flash-lisp.scm [--dry-run] [--openocd-root DIR] [--interface CFG] [--target CFG]")
  (say "")
  (say "Flashes the bootloader image that contains lisp-psoc-pc."))

(define (parse args dry-run? openocd-root interface target)
  (cond
    ((null? args) (values dry-run? openocd-root interface target))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--dry-run")
     (parse (cdr args) #t openocd-root interface target))
    ((string=? (car args) "--openocd-root")
     (if (null? (cdr args))
         (die "--openocd-root needs a directory")
         (parse (cddr args) dry-run? (cadr args) interface target)))
    ((string=? (car args) "--interface")
     (if (null? (cdr args))
         (die "--interface needs an OpenOCD cfg path")
         (parse (cddr args) dry-run? openocd-root (cadr args) target)))
    ((string=? (car args) "--target")
     (if (null? (cdr args))
         (die "--target needs an OpenOCD cfg path")
         (parse (cddr args) dry-run? openocd-root interface (cadr args))))
    (else
     (die (string-append "unknown argument: " (car args))))))

(define-values (dry-run? openocd-root interface target)
  (parse (command-line-tail)
         #f
         #f
         (or (env "PSOC_OPENOCD_INTERFACE") "interface/cmsis-dap.cfg")
         (or (env "PSOC_OPENOCD_TARGET") "target/psoc6_2m.cfg")))

(define root (or openocd-root (ensure-openocd-root)))
(define image
  "psoc6-cm0-bootloader/target/thumbv6m-none-eabi/release/psoc6-cm0-bootloader")

(unless (file-exists? (repo-path image))
  (die "bootloader image is missing; run tools/build-lisp.scm first"))

(define command
  (string-append
   (shell-quote (path-join root "bin/openocd"))
   " -s "
   (shell-quote (path-join root "scripts"))
   " -f "
   (shell-quote interface)
   " -f "
   (shell-quote target)
   " -c "
   (shell-quote (string-append "program " image " verify reset exit"))))

(if dry-run?
    (say command)
    (run-in (repo-root) command))
