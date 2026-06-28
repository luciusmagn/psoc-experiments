(define tool-versions
  '("3.8" "3.7" "3.6" "3.5" "3.4" "3.3" "3.2" "3.1"))

(define psoc-serial-baud "115200")

(define (say text)
  (display text)
  (newline))

(define (die text)
  (display "error: " (current-error-port))
  (display text (current-error-port))
  (newline (current-error-port))
  (exit 1))

(define (last-slash text)
  (let loop ((index (- (string-length text) 1)))
    (cond
      ((< index 0) #f)
      ((char=? (string-ref text index) #\/) index)
      (else (loop (- index 1))))))

(define (dirname path)
  (let ((slash (last-slash path)))
    (cond
      ((not slash) ".")
      ((= slash 0) "/")
      (else (substring path 0 slash)))))

(define (path-join left right)
  (cond
    ((string=? left "") right)
    ((string=? right "") left)
    ((char=? (string-ref right 0) #\/) right)
    ((char=? (string-ref left (- (string-length left) 1)) #\/)
     (string-append left right))
    (else (string-append left "/" right))))

(define (script-dir)
  (dirname (car (command-line))))

(define (repo-root)
  (dirname (script-dir)))

(define (repo-path path)
  (path-join (repo-root) path))

(define (shell-quote text)
  (let ((port (open-output-string)))
    (put-char port #\')
    (let loop ((index 0))
      (when (< index (string-length text))
        (let ((char (string-ref text index)))
          (if (char=? char #\')
              (display "'\\''" port)
              (put-char port char)))
        (loop (+ index 1))))
    (put-char port #\')
    (get-output-string port)))

(define (run command)
  (display "+ ")
  (say command)
  (let ((status (system command)))
    (unless (zero? status)
      (die (string-append "command failed with status "
                          (number->string status)))))
  0)

(define (run-in directory command)
  (run (string-append "cd " (shell-quote directory) " && " command)))

(define (env name)
  (let ((value (getenv name)))
    (and value (not (string=? value "")) value)))

(define (maybe-list value)
  (if value (list value) '()))

(define (openocd-root-valid? root)
  (and root
       (file-exists? (path-join root "bin/openocd"))
       (file-exists? (path-join root "scripts"))))

(define (openocd-roots-under base)
  (let loop ((versions tool-versions) (roots '()))
    (if (null? versions)
        (reverse roots)
        (loop (cdr versions)
              (cons (path-join base
                               (string-append "tools_"
                                              (car versions)
                                              "/openocd"))
                    roots)))))

(define (candidate-openocd-roots)
  (append
   (maybe-list (env "OPENOCD_ROOT"))
   (maybe-list (env "MODUSTOOLBOX_OPENOCD_ROOT"))
   (if (env "MODUSTOOLBOX_ROOT")
       (openocd-roots-under (env "MODUSTOOLBOX_ROOT"))
       '())
   (openocd-roots-under (repo-path ".local/ModusToolbox"))
   (openocd-roots-under "/opt/ModusToolbox")))

(define (find-openocd-root)
  (let loop ((roots (candidate-openocd-roots)))
    (cond
      ((null? roots) #f)
      ((openocd-root-valid? (car roots)) (car roots))
      (else (loop (cdr roots))))))

(define (ensure-openocd-root)
  (let ((root (find-openocd-root)))
    (if root
        root
        (die "no Infineon OpenOCD found; set OPENOCD_ROOT or run tools/setup-modustoolbox.scm"))))

(define (ensure-local-dir)
  (run (string-append "mkdir -p " (shell-quote (repo-path ".local")))))

(define (configure-serial-device device)
  (run (string-append
        "stty -F "
        (shell-quote device)
        " "
        psoc-serial-baud
        " cs8 -cstopb -parenb -ixon -ixoff -crtscts -hupcl clocal raw -echo -echoe -echok -echoctl -echoke -icanon min 1 time 0")))

(define (capture-first-line command)
  (ensure-local-dir)
  (let ((path (repo-path ".local/psoc-tools-capture.txt")))
    (let ((status (system (string-append command " > " (shell-quote path)))))
      (unless (zero? status)
        (die (string-append "capture command failed with status "
                            (number->string status)))))
    (if (file-exists? path)
        (call-with-input-file path
          (lambda (port)
            (let ((line (get-line port)))
              (if (eof-object? line) "" line))))
        "")))

(define (string-prefix? prefix text)
  (and (<= (string-length prefix) (string-length text))
       (string=? prefix (substring text 0 (string-length prefix)))))

(define (string-contains-char? text wanted)
  (let loop ((index 0))
    (cond
      ((>= index (string-length text)) #f)
      ((char=? (string-ref text index) wanted) index)
      (else (loop (+ index 1))))))

(define (first-path-component path)
  (let ((slash (string-contains-char? path #\/)))
    (if slash (substring path 0 slash) path)))

(define (command-line-tail)
  (cdr (command-line)))
