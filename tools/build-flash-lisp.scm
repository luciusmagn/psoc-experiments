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

(let ((args (command-line-tail)))
  (when (and (pair? args) (string=? (car args) "--help"))
    (say "usage: tools/build-flash-lisp.scm [flash-lisp arguments]")
    (say "")
    (say "Builds lisp-psoc-pc, packs the bootloader, then flashes it.")
    (exit 0))
  (run (shell-quote (repo-path "tools/build-lisp.scm")))
  (run (string-append
        (shell-quote (repo-path "tools/flash-lisp.scm"))
        (let ((rest (join-with-spaces (quote-args args))))
          (if (string=? rest "") "" (string-append " " rest))))))
