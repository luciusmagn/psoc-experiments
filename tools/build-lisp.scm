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
  (say "usage: tools/build-lisp.scm")
  (say "")
  (say "Builds the CM4 Lisp firmware, packs it into the CM0+ bootloader,")
  (say "and rebuilds the bootloader image."))

(let ((args (command-line-tail)))
  (when (and (pair? args) (string=? (car args) "--help"))
    (usage)
    (exit 0))
  (unless (null? args)
    (die "build-lisp.scm takes no arguments")))

(run-in (repo-path "lisp-psoc-pc")
        "RUSTFLAGS=-Awarnings cargo build --release --features use-bootloader")
(run-in (repo-path "lisp-psoc-pc")
        "arm-none-eabi-objcopy -O binary target/thumbv7em-none-eabihf/release/lisp-psoc-pc ../psoc6-cm0-bootloader/src/app.bin")
(run-in (repo-path "psoc6-cm0-bootloader")
        "RUSTFLAGS=-Awarnings cargo build --release")
