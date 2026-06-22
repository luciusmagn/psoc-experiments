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
  (say "usage: tools/setup-modustoolbox.scm [--check] [--prefix DIR] [--archive TAR] [--url URL]")
  (say "")
  (say "Installs or checks the repo-local ModusToolbox tools directory.")
  (say "Downloaded or extracted files live under .local/ModusToolbox by default.")
  (say "If Infineon requires browser login, download the Linux tarball manually")
  (say "and pass it with --archive."))

(define (parse args prefix archive url check?)
  (cond
    ((null? args) (values prefix archive url check?))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--check")
     (parse (cdr args) prefix archive url #t))
    ((string=? (car args) "--prefix")
     (if (null? (cdr args))
         (die "--prefix needs a directory")
         (parse (cddr args) (cadr args) archive url check?)))
    ((string=? (car args) "--archive")
     (if (null? (cdr args))
         (die "--archive needs a tarball path")
         (parse (cddr args) prefix (cadr args) url check?)))
    ((string=? (car args) "--url")
     (if (null? (cdr args))
         (die "--url needs a URL")
         (parse (cddr args) prefix archive (cadr args) check?)))
    (else
     (die (string-append "unknown argument: " (car args))))))

(define (show-openocd-root)
  (let ((root (find-openocd-root)))
    (if root
        (begin
          (say (string-append "OPENOCD_ROOT=" root))
          (say (string-append "openocd=" (path-join root "bin/openocd")))
          (say (string-append "scripts=" (path-join root "scripts")))
          #t)
        (begin
          (say "OPENOCD_ROOT not found")
          #f))))

(define (download-url url)
  (let ((target (repo-path ".local/downloads/modustoolbox-linux.tar.gz")))
    (run (string-append "mkdir -p "
                        (shell-quote (dirname target))))
    (run (string-append "curl --fail --location --output "
                        (shell-quote target)
                        " "
                        (shell-quote url)))
    target))

(define (strip-components? archive)
  (let* ((first-entry
          (capture-first-line
           (string-append "tar -tf " (shell-quote archive) " | sed -n '1p'")))
         (top (first-path-component first-entry)))
    (or (string-prefix? "ModusToolbox" top)
        (string-prefix? "modus-toolbox" top)
        (string-prefix? "mtb-" top))))

(define (extract-archive archive prefix)
  (unless (file-exists? archive)
    (die (string-append "archive not found: " archive)))
  (run (string-append "mkdir -p " (shell-quote prefix)))
  (run (string-append "tar -xf "
                      (shell-quote archive)
                      " -C "
                      (shell-quote prefix)
                      (if (strip-components? archive)
                          " --strip-components=1"
                          ""))))

(define (find-openocd-under prefix)
  (capture-first-line
   (string-append "find "
                  (shell-quote prefix)
                  " -path '*/openocd/bin/openocd' -type f -print -quit | sed 's#/bin/openocd$##'")))

(define-values (prefix archive url check?)
  (parse (command-line-tail)
         (repo-path ".local/ModusToolbox")
         #f
         #f
         #f))

(cond
  ((and check? (not archive) (not url))
   (unless (show-openocd-root)
     (exit 1)))
  (else
   (let ((selected-archive (if url (download-url url) archive)))
     (if selected-archive
         (begin
           (extract-archive selected-archive prefix)
           (let ((root (find-openocd-under prefix)))
             (if (string=? root "")
                 (begin
                   (say "installed archive, but no openocd/bin/openocd was found under prefix")
                   (show-openocd-root)
                   (exit 1))
                 (begin
                   (say (string-append "installed OPENOCD_ROOT=" root))
                   (say "use this for the current shell if needed:")
                   (say (string-append "export OPENOCD_ROOT=" (shell-quote root)))))))
         (begin
           (unless (show-openocd-root)
             (usage)
             (exit 1)))))))
