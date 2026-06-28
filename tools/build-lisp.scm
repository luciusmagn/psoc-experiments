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
  (say "usage: tools/build-lisp.scm [--wifi-firmware] [--wifi-credentials] [--wifi-boot-smoke]")
  (say "")
  (say "Builds the CM4 Lisp firmware, packs it into the CM0+ bootloader,")
  (say "and rebuilds the bootloader image.")
  (say "")
  (say "--wifi-firmware includes local firmware, NVRAM, and CLM resources in the CM4 image.")
  (say "--wifi-credentials includes ignored local SSID/passphrase blobs in the CM4 image.")
  (say "--wifi-boot-smoke runs local connect and link-status forms at boot.")
  (say "  It implies --wifi-firmware and --wifi-credentials."))

(define (parse args wifi-firmware? wifi-credentials? wifi-boot-smoke?)
  (cond
    ((null? args) (values wifi-firmware? wifi-credentials? wifi-boot-smoke?))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--wifi-firmware")
     (parse (cdr args) #t wifi-credentials? wifi-boot-smoke?))
    ((string=? (car args) "--wifi-credentials")
     (parse (cdr args) wifi-firmware? #t wifi-boot-smoke?))
    ((string=? (car args) "--wifi-boot-smoke")
     (parse (cdr args) #t #t #t))
    (else
     (die (string-append "unknown argument: " (car args))))))

(define (join-with-comma items)
  (let loop ((items items) (out ""))
    (cond
      ((null? items) out)
      ((string=? out "") (loop (cdr items) (car items)))
      (else (loop (cdr items) (string-append out "," (car items)))))))

(define (feature-list wifi-firmware? wifi-credentials? wifi-boot-smoke?)
  (join-with-comma
   (append
    '("use-bootloader")
    (if wifi-firmware? '("wifi-firmware-blob") '())
    (if wifi-credentials? '("wifi-local-credentials") '())
    (if wifi-boot-smoke? '("wifi-boot-smoke") '()))))

(define-values (wifi-firmware? wifi-credentials? wifi-boot-smoke?)
  (parse (command-line-tail) #f #f #f))
(define features (feature-list wifi-firmware? wifi-credentials? wifi-boot-smoke?))

(when wifi-firmware?
  (run (string-append
        (shell-quote (repo-path "tools/prepare-wifi-resources.scm"))
        " --check")))
(when wifi-credentials?
  (run (shell-quote (repo-path "tools/prepare-wifi-credential-blobs.scm"))))

(run-in (repo-path "lisp-psoc-pc")
        (string-append
         "RUSTFLAGS=-Awarnings cargo build --release --features "
         (shell-quote features)))
(run-in (repo-path "lisp-psoc-pc")
        "arm-none-eabi-objcopy -O binary target/thumbv7em-none-eabihf/release/lisp-psoc-pc ../psoc6-cm0-bootloader/src/app.bin")
(run-in (repo-path "psoc6-cm0-bootloader")
        "RUSTFLAGS=-Awarnings cargo build --release")
