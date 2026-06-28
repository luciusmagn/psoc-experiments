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

(define default-port 4665)
(define default-sequence 1)
(define default-attempts 5)
(define default-wait-seconds 1)
(define max-request-bytes 96)
(define fnv-offset-basis #x811c9dc5)
(define fnv-prime #x01000193)
(define u32-modulus #x100000000)
(define request-path ".local/net-repl/request.bin")
(define response-path ".local/net-repl/response.bin")

(define (usage)
  (say "usage: tools/send-net-repl.scm --host HOST [--port PORT] [--sequence N] [--attempts N] [--wait SECONDS] [--color] [--read-only] FORM")
  (say "")
  (say "Sends one framed UDP Lisp request to the board network REPL endpoint.")
  (say "The Lisp form is written to an ignored binary request file, not printed in the shell command.")
  (say "--color wraps the payload text in ANSI color. The default is plain output.")
  (say "--read-only refuses forms outside the conservative host-side read-only allowlist.")
  (say "HOST may also be supplied with PSOC_NET_REPL_HOST."))

(define (parse-integer-option name text min max)
  (let ((value (string->number text)))
    (if (and value (integer? value) (exact? value) (>= value min) (<= value max))
        value
        (die (string-append name " needs an integer in range "
                            (number->string min)
                            ".."
                            (number->string max))))))

(define (parse args host port sequence attempts wait-seconds color? read-only? forms)
  (cond
    ((null? args)
     (values host port sequence attempts wait-seconds color? read-only? (reverse forms)))
    ((string=? (car args) "--help")
     (usage)
     (exit 0))
    ((string=? (car args) "--host")
     (if (null? (cdr args))
         (die "--host needs a value")
         (parse (cddr args)
                (cadr args)
                port
                sequence
                attempts
                wait-seconds
                color?
                read-only?
                forms)))
    ((string=? (car args) "--port")
     (if (null? (cdr args))
         (die "--port needs a value")
         (parse (cddr args)
                host
                (parse-integer-option "--port" (cadr args) 1 65535)
                sequence
                attempts
                wait-seconds
                color?
                read-only?
                forms)))
    ((string=? (car args) "--sequence")
     (if (null? (cdr args))
         (die "--sequence needs a value")
         (parse (cddr args)
                host
                port
                (parse-integer-option "--sequence" (cadr args) 0 4294967295)
                attempts
                wait-seconds
                color?
                read-only?
                forms)))
    ((string=? (car args) "--attempts")
     (if (null? (cdr args))
         (die "--attempts needs a value")
         (parse (cddr args)
                host
                port
                sequence
                (parse-integer-option "--attempts" (cadr args) 1 1000)
                wait-seconds
                color?
                read-only?
                forms)))
    ((string=? (car args) "--wait")
     (if (null? (cdr args))
         (die "--wait needs a value")
         (parse (cddr args)
                host
                port
                sequence
                attempts
                (parse-integer-option "--wait" (cadr args) 1 60)
                color?
                read-only?
                forms)))
    ((string=? (car args) "--color")
     (parse (cdr args) host port sequence attempts wait-seconds #t read-only? forms))
    ((string=? (car args) "--no-color")
     (parse (cdr args) host port sequence attempts wait-seconds #f read-only? forms))
    ((string=? (car args) "--read-only")
     (parse (cdr args) host port sequence attempts wait-seconds color? #t forms))
    (else
     (parse (cdr args)
            host
            port
            sequence
            attempts
            wait-seconds
            color?
            read-only?
            (cons (car args) forms)))))

(define (join-form parts)
  (let loop ((items parts) (out ""))
    (cond
      ((null? items) out)
      ((string=? out "") (loop (cdr items) (car items)))
      (else (loop (cdr items) (string-append out " " (car items)))))))

(define (put-ascii port text)
  (let loop ((index 0))
    (when (< index (string-length text))
      (let ((byte (char->integer (string-ref text index))))
        (when (> byte 255)
          (die "form contains a non-byte character"))
        (put-u8 port byte))
      (loop (+ index 1)))))

(define (put-u32-be port value)
  (put-u8 port (bitwise-and (bitwise-arithmetic-shift-right value 24) #xff))
  (put-u8 port (bitwise-and (bitwise-arithmetic-shift-right value 16) #xff))
  (put-u8 port (bitwise-and (bitwise-arithmetic-shift-right value 8) #xff))
  (put-u8 port (bitwise-and value #xff)))

(define (read-u32-be bytes offset)
  (+ (bitwise-arithmetic-shift-left (list-ref bytes offset) 24)
     (bitwise-arithmetic-shift-left (list-ref bytes (+ offset 1)) 16)
     (bitwise-arithmetic-shift-left (list-ref bytes (+ offset 2)) 8)
     (list-ref bytes (+ offset 3))))

(define (write-request path sequence form)
  (run (string-append "mkdir -p " (shell-quote (dirname path))))
  (call-with-port
   (open-file-output-port path (file-options no-fail replace) (buffer-mode none))
   (lambda (port)
     (put-ascii port "LPS0")
     (put-u32-be port sequence)
     (put-ascii port form)))
  (run (string-append "chmod 600 " (shell-quote path))))

(define (read-bytes path)
  (if (file-exists? path)
      (call-with-port
       (open-file-input-port path)
       (lambda (port)
         (let loop ((bytes '()))
           (let ((byte (get-u8 port)))
             (if (eof-object? byte)
                 (reverse bytes)
                 (loop (cons byte bytes)))))))
      '()))

(define (hex-digit value)
  (string-ref "0123456789abcdef" value))

(define (byte-hex byte)
  (let ((text (make-string 2)))
    (string-set! text 0 (hex-digit (quotient byte 16)))
    (string-set! text 1 (hex-digit (modulo byte 16)))
    text))

(define (bytes-hex bytes)
  (let loop ((items bytes) (out ""))
    (cond
      ((null? items) out)
      ((string=? out "") (loop (cdr items) (byte-hex (car items))))
      (else (loop (cdr items) (string-append out " " (byte-hex (car items))))))))

(define (u32-hex value)
  (let ((text (make-string 8)))
    (let loop ((index 7) (remaining value))
      (when (>= index 0)
        (string-set! text index (hex-digit (modulo remaining 16)))
        (loop (- index 1) (quotient remaining 16))))
    text))

(define (checksum-bytes bytes)
  (let loop ((items bytes) (hash fnv-offset-basis))
    (if (null? items)
        hash
        (loop (cdr items)
              (modulo (* (bitwise-xor hash (car items)) fnv-prime)
                      u32-modulus)))))

(define (write-ansi code)
  (put-char (current-output-port) (integer->char 27))
  (display "[")
  (display code)
  (display "m"))

(define (bytes-prefix-ascii? prefix bytes)
  (and (<= (string-length prefix) (length bytes))
       (let loop ((index 0) (items bytes))
         (cond
           ((= index (string-length prefix)) #t)
           ((null? items) #f)
           ((= (char->integer (string-ref prefix index)) (car items))
            (loop (+ index 1) (cdr items)))
           (else #f)))))

(define (payload-color-code bytes)
  (cond
    ((bytes-prefix-ascii? "=> " bytes) "32")
    ((bytes-prefix-ascii? "error:" bytes) "31")
    (else "36")))

(define (bytes-end-with? wanted bytes)
  (and (pair? bytes)
       (let loop ((items bytes))
         (if (null? (cdr items))
             (= (car items) wanted)
             (loop (cdr items))))))

(define (drop-final-byte bytes)
  (let loop ((items bytes) (out '()))
    (cond
      ((null? items) (reverse out))
      ((null? (cdr items)) (reverse out))
      (else (loop (cdr items) (cons (car items) out))))))

(define (display-payload bytes)
  (let loop ((items bytes))
    (when (pair? items)
      (let ((byte (car items)))
        (cond
          ((= byte 10) (newline))
          ((= byte 13) (display "\\r"))
          ((= byte 9) (display "\t"))
          ((and (>= byte 32) (<= byte 126)) (put-char (current-output-port) (integer->char byte)))
          (else
           (display "\\x")
           (display (byte-hex byte)))))
      (loop (cdr items)))))

(define (display-payload-line bytes color?)
  (let ((payload (if (bytes-end-with? 10 bytes)
                     (drop-final-byte bytes)
                     bytes)))
    (display "payload.text=")
    (when color?
      (write-ansi (payload-color-code bytes)))
    (display-payload payload)
    (when color?
      (write-ansi "0"))
    (newline)))

(define (ascii-space? char)
  (or (char=? char #\space)
      (char=? char #\tab)
      (char=? char #\newline)
      (char=? char #\return)))

(define (trim-ascii text)
  (let find-start ((start 0))
    (if (and (< start (string-length text))
             (ascii-space? (string-ref text start)))
        (find-start (+ start 1))
        (let find-end ((end (string-length text)))
          (if (and (> end start)
                   (ascii-space? (string-ref text (- end 1))))
              (find-end (- end 1))
              (substring text start end))))))

(define (string-suffix? suffix text)
  (let ((text-len (string-length text))
        (suffix-len (string-length suffix)))
    (and (<= suffix-len text-len)
         (string=? suffix (substring text (- text-len suffix-len) text-len)))))

(define (ascii-alphanumeric? char)
  (or (and (char>=? char #\a) (char<=? char #\z))
      (and (char>=? char #\A) (char<=? char #\Z))
      (and (char>=? char #\0) (char<=? char #\9))))

(define (safe-read-path-char? char)
  (or (ascii-alphanumeric? char)
      (char=? char #\.)
      (char=? char #\-)
      (char=? char #\_)
      (char=? char #\/)))

(define (safe-read-path? text)
  (and (> (string-length text) 0)
       (let loop ((index 0))
         (cond
           ((>= index (string-length text)) #t)
           ((safe-read-path-char? (string-ref text index))
            (loop (+ index 1)))
           (else #f)))))

(define (safe-read-string-call? prefix text)
  (let ((prefix-len (string-length prefix))
        (text-len (string-length text)))
    (and (string-prefix? prefix text)
         (string-suffix? "\")" text)
         (> text-len (+ prefix-len 2))
         (safe-read-path? (substring text prefix-len (- text-len 2))))))

(define (read-only-form? form)
  (let ((text (trim-ascii form)))
    (or (string=? text "(wifi-net-repl-service status)")
        (string=? text "(ls)")
        (string=? text "(fat-info)")
        (string=? text "(sd-status)")
        (safe-read-string-call? "(cat \"" text)
        (safe-read-string-call? "(read-file \"" text))))

(define (enforce-read-only form)
  (unless (read-only-form? form)
    (die "read-only mode allows only status, ls, fat-info, sd-status, cat, and read-file forms")))

(define (legacy-response? bytes sequence)
  (and (>= (length bytes) 8)
       (= (list-ref bytes 0) (char->integer #\L))
       (= (list-ref bytes 1) (char->integer #\P))
       (= (list-ref bytes 2) (char->integer #\S))
       (= (list-ref bytes 3) (char->integer #\1))
       (= (read-u32-be bytes 4) sequence)))

(define (checked-response? bytes sequence)
  (and (>= (length bytes) 12)
       (= (list-ref bytes 0) (char->integer #\L))
       (= (list-ref bytes 1) (char->integer #\P))
       (= (list-ref bytes 2) (char->integer #\S))
       (= (list-ref bytes 3) (char->integer #\2))
       (= (read-u32-be bytes 4) sequence)))

(define (response-header-bytes bytes)
  (if (and (>= (length bytes) 4)
           (= (list-ref bytes 0) (char->integer #\L))
           (= (list-ref bytes 1) (char->integer #\P))
           (= (list-ref bytes 2) (char->integer #\S))
           (= (list-ref bytes 3) (char->integer #\2)))
      12
      8))

(define (take-after-count bytes count)
  (let loop ((items bytes) (index 0))
    (cond
      ((null? items) '())
      ((< index count) (loop (cdr items) (+ index 1)))
      (else items))))

(define (take-after-header bytes)
  (take-after-count bytes (response-header-bytes bytes)))

(define (response-checksum bytes)
  (if (checked-response? bytes (read-u32-be bytes 4))
      (read-u32-be bytes 8)
      #f))

(define (response-checksum-valid? bytes sequence)
  (cond
    ((checked-response? bytes sequence)
     (= (read-u32-be bytes 8) (checksum-bytes (take-after-header bytes))))
    ((legacy-response? bytes sequence) #t)
    (else #f)))

(define (valid-response? bytes sequence)
  (and (or (checked-response? bytes sequence)
           (legacy-response? bytes sequence))
       (response-checksum-valid? bytes sequence)))

(define (response-format bytes)
  (if (= (response-header-bytes bytes) 12) "LPS2" "LPS1"))

(define (send-attempt host port wait-seconds request response)
  (when (file-exists? response)
    (delete-file response))
  (let* ((process-timeout-seconds (+ wait-seconds 1))
         (command
         (string-append
          "timeout "
          (number->string process-timeout-seconds)
          " nc -u -w "
          (number->string wait-seconds)
          " "
          (shell-quote host)
          " "
          (number->string port)
          " < "
          (shell-quote request)
          " > "
          (shell-quote response))))
    (display "+ ")
    (say command)
    (system command)))

(define-values (host port sequence attempts wait-seconds color? read-only? parts)
  (parse (command-line-tail)
         (env "PSOC_NET_REPL_HOST")
         default-port
         default-sequence
         default-attempts
         default-wait-seconds
         #f
         #f
         '()))

(when (not host)
  (usage)
  (die "--host or PSOC_NET_REPL_HOST is required"))

(when (null? parts)
  (usage)
  (exit 1))

(define form (join-form parts))
(when (> (string-length form) max-request-bytes)
  (die (string-append "form is longer than "
                      (number->string max-request-bytes)
                      " bytes")))

(when read-only?
  (enforce-read-only form))

(define request-file (repo-path request-path))
(define response-file (repo-path response-path))

(write-request request-file sequence form)
(say (string-append "request.bytes=" (number->string (+ 8 (string-length form)))))
(say (string-append "sequence=" (number->string sequence)))

(let loop ((attempt 1))
  (send-attempt host port wait-seconds request-file response-file)
  (let ((response (read-bytes response-file)))
    (cond
      ((valid-response? response sequence)
       (let ((payload (take-after-header response)))
         (say (string-append "attempt=" (number->string attempt)))
         (say (string-append "response.bytes=" (number->string (length response))))
         (say (string-append "response.format=" (response-format response)))
         (say (string-append "response.hex=" (bytes-hex response)))
         (let ((expected-checksum (response-checksum response)))
           (when expected-checksum
             (say (string-append "response.checksum=#x"
                                 (u32-hex expected-checksum)))
             (say (string-append "response.checksum.ok="
                                 (if (response-checksum-valid? response sequence)
                                     "#t"
                                     "#f")))))
         (say (string-append "payload.bytes=" (number->string (length payload))))
         (display-payload-line payload color?)
         (exit 0)))
      ((>= attempt attempts)
       (if (null? response)
           (die "no response")
           (die (string-append "invalid response: " (bytes-hex response)))))
      (else
       (loop (+ attempt 1))))))
