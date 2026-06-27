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
  (say "usage: tools/build-lisp.scm [--wifi-firmware]")
  (say "")
  (say "Builds the CM4 Lisp firmware, packs it into the CM0+ bootloader,")
  (say "and rebuilds the bootloader image.")
  (say "")
  (say "--wifi-firmware includes local firmware, NVRAM, and CLM resources in the CM4 image."))

(define (parse args wifi-firmware?)
  (cond
    ((null? args) wifi-firmware?)
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--wifi-firmware")
     (parse (cdr args) #t))
    (else
     (die (string-append "unknown argument: " (car args))))))

(define wifi-firmware? (parse (command-line-tail) #f))
(define features
  (if wifi-firmware?
      "use-bootloader,wifi-firmware-blob"
      "use-bootloader"))

(when wifi-firmware?
  (run (string-append
        (shell-quote (repo-path "tools/prepare-wifi-resources.scm"))
        " --check")))

(run-in (repo-path "lisp-psoc-pc")
        (string-append
         "RUSTFLAGS=-Awarnings cargo build --release --features "
         (shell-quote features)))
(run-in (repo-path "lisp-psoc-pc")
        "arm-none-eabi-objcopy -O binary target/thumbv7em-none-eabihf/release/lisp-psoc-pc ../psoc6-cm0-bootloader/src/app.bin")
(run-in (repo-path "psoc6-cm0-bootloader")
        "RUSTFLAGS=-Awarnings cargo build --release")
