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

(define (quote-args args)
  (let loop ((items args) (parts '()))
    (if (null? items)
        (reverse parts)
        (loop (cdr items) (cons (shell-quote (car items)) parts)))))

(define (join-with-spaces items)
  (let loop ((items items) (out ""))
    (cond
      ((null? items) out)
      ((string=? out "") (loop (cdr items) (car items)))
      (else (loop (cdr items) (string-append out " " (car items)))))))

(define (parse args build-args flash-args)
  (cond
    ((null? args) (values (reverse build-args) (reverse flash-args)))
    ((or (string=? (car args) "--wifi-firmware")
         (string=? (car args) "--wifi-credentials")
         (string=? (car args) "--wifi-boot-smoke")
         (string=? (car args) "--wifi-dhcp-boot-smoke")
         (string=? (car args) "--wifi-arp-boot-smoke")
         (string=? (car args) "--wifi-dns-boot-smoke")
         (string=? (car args) "--storage-boot-smoke")
         (string=? (car args) "--storage-format-boot-smoke"))
     (parse (cdr args) (cons (car args) build-args) flash-args))
    (else
     (parse (cdr args) build-args (cons (car args) flash-args)))))

(define (command-with-args command args)
  (let ((rest (join-with-spaces (quote-args args))))
    (if (string=? rest "")
        command
        (string-append command " " rest))))

(let ((args (command-line-tail)))
  (when (and (pair? args) (string=? (car args) "--help"))
    (say "usage: tools/build-flash-lisp.scm [--wifi-firmware] [--wifi-credentials] [--wifi-boot-smoke] [--wifi-dhcp-boot-smoke] [--wifi-arp-boot-smoke] [--wifi-dns-boot-smoke] [--storage-boot-smoke] [--storage-format-boot-smoke] [flash-lisp arguments]")
    (say "")
    (say "Builds lisp-psoc-pc, packs the bootloader, then flashes it.")
    (exit 0))
  (let-values (((build-args flash-args) (parse args '() '())))
    (run (command-with-args
          (shell-quote (repo-path "tools/build-lisp.scm"))
          build-args))
    (run (command-with-args
          (shell-quote (repo-path "tools/flash-lisp.scm"))
          flash-args))))
