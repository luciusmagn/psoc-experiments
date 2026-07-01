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
(define net-repl-state-path ".local/net-repl")
(define client-run-id (number->string (get-process-id)))
(define client-state-path (path-join net-repl-state-path client-run-id))
(define request-path (path-join client-state-path "request.bin"))
(define response-path (path-join client-state-path "response.bin"))
(define ack-path (path-join client-state-path "ack.bin"))

(define (usage)
  (say "usage: tools/send-net-repl.scm --host HOST [--port PORT] [--sequence N] [--attempts N] [--wait SECONDS] [--color] [--payload-only] [--read-only] [--legacy-request] FORM")
  (say "")
  (say "Sends one framed UDP Lisp request to the board network REPL endpoint.")
  (say "The Lisp form is written to an ignored binary request file, not printed in the shell command.")
  (say "The client uses ncat when available to receive larger UDP datagrams, falling back to nc.")
  (say "The default request frame is LPS3 with a request checksum.")
  (say "--read-only sends LPS5 with the same checksum and asks current firmware to reject mutating forms.")
  (say "After a verified response, the client sends an LPS4 ACK with the response checksum.")
  (say "--legacy-request sends the older LPS0 request frame for older flashed images.")
  (say "--color wraps the payload text in ANSI color. The default is plain output.")
  (say "--payload-only prints only the decoded response payload.")
  (say "--read-only also refuses forms outside the conservative host-side read-only allowlist.")
  (say "HOST may also be supplied with PSOC_NET_REPL_HOST."))

(define (parse-integer-option name text min max)
  (let ((value (string->number text)))
    (if (and value (integer? value) (exact? value) (>= value min) (<= value max))
        value
        (die (string-append name " needs an integer in range "
                            (number->string min)
                            ".."
                            (number->string max))))))

(define (parse args host port sequence attempts wait-seconds color? read-only? legacy-request? payload-only? forms)
  (cond
    ((null? args)
     (values host port sequence attempts wait-seconds color? read-only? legacy-request? payload-only? (reverse forms)))
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
                legacy-request?
                payload-only?
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
                legacy-request?
                payload-only?
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
                legacy-request?
                payload-only?
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
                legacy-request?
                payload-only?
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
                legacy-request?
                payload-only?
                forms)))
    ((string=? (car args) "--color")
     (parse (cdr args) host port sequence attempts wait-seconds #t read-only? legacy-request? payload-only? forms))
    ((string=? (car args) "--no-color")
     (parse (cdr args) host port sequence attempts wait-seconds #f read-only? legacy-request? payload-only? forms))
    ((string=? (car args) "--payload-only")
     (parse (cdr args) host port sequence attempts wait-seconds color? read-only? legacy-request? #t forms))
    ((string=? (car args) "--read-only")
     (parse (cdr args) host port sequence attempts wait-seconds color? #t legacy-request? payload-only? forms))
    ((string=? (car args) "--legacy-request")
     (parse (cdr args) host port sequence attempts wait-seconds color? read-only? #t payload-only? forms))
    (else
     (parse (cdr args)
            host
            port
            sequence
            attempts
            wait-seconds
            color?
            read-only?
            legacy-request?
            payload-only?
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

(define (ascii-byte-list text)
  (let loop ((index 0) (bytes '()))
    (if (= index (string-length text))
        (reverse bytes)
        (let ((byte (char->integer (string-ref text index))))
          (when (> byte 255)
            (die "form contains a non-byte character"))
          (loop (+ index 1) (cons byte bytes))))))

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

(define (run-checked command verbose?)
  (when verbose?
    (display "+ ")
    (say command))
  (let ((status (system command)))
    (unless (zero? status)
      (die (string-append "command failed with status "
                          (number->string status)))))
  0)

(define (run-unchecked command verbose?)
  (when verbose?
    (display "+ ")
    (say command))
  (system command))

(define (request-magic legacy-request? read-only?)
  (cond
    (legacy-request? "LPS0")
    (read-only? "LPS5")
    (else "LPS3")))

(define (write-request path sequence form legacy-request? read-only? verbose?)
  (run-checked (string-append "mkdir -p " (shell-quote (dirname path))) verbose?)
  (call-with-port
   (open-file-output-port path (file-options no-fail replace) (buffer-mode none))
   (lambda (port)
     (put-ascii port (request-magic legacy-request? read-only?))
     (put-u32-be port sequence)
     (when (not legacy-request?)
       (put-u32-be port (checksum-bytes (ascii-byte-list form))))
     (put-ascii port form)))
  (run-checked (string-append "chmod 600 " (shell-quote path)) verbose?))

(define (write-ack path sequence response-hash verbose?)
  (run-checked (string-append "mkdir -p " (shell-quote (dirname path))) verbose?)
  (call-with-port
   (open-file-output-port path (file-options no-fail replace) (buffer-mode none))
   (lambda (port)
     (put-ascii port "LPS4")
     (put-u32-be port sequence)
     (put-u32-be port response-hash)))
  (run-checked (string-append "chmod 600 " (shell-quote path)) verbose?))

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

(define (display-payload-raw bytes color?)
  (when color?
    (write-ansi (payload-color-code bytes)))
  (display-payload bytes)
  (when color?
    (write-ansi "0"))
  (unless (bytes-end-with? 10 bytes)
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
        (string=? text "(wifi-tcp-repl-service status)")
        (string=? text "(wifi-demux-status)")
        (string=? text "(processes)")
        (string=? text "(ls)")
        (string=? text "(fat-info)")
        (string=? text "(sd-status)")
        (string=? text "(wifi-link-status)")
        (string=? text "(wifi-lease-status)")
        (string=? text "(help)")
        (string=? text "(millis)")
        (string=? text "(regs)")
        (string=? text "(heap)")
        (string=? text "(pdm-status)")
        (string=? text "(thermistor-status)")
        (string=? text "(capsense-status)")
        (safe-read-string-call? "(cat \"" text)
        (safe-read-string-call? "(read-file \"" text))))

(define (enforce-read-only form)
  (unless (read-only-form? form)
    (die "read-only mode allows only status, processes, help, millis, regs, heap, ls, fat-info, sd-status, Wi-Fi link/lease status, demux status, board status, network REPL service status, cat, and read-file forms")))

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

(define (response-payload-checksum bytes)
  (checksum-bytes (take-after-header bytes)))

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

(define (send-attempt host port wait-seconds request response verbose?)
  (when (file-exists? response)
    (delete-file response))
  (let* ((process-timeout-seconds (+ wait-seconds 1))
         (command
         (string-append
          "if command -v ncat >/dev/null 2>&1; then timeout "
          (number->string process-timeout-seconds)
          " ncat -u -w "
          (number->string wait-seconds)
          " -q "
          (number->string wait-seconds)
          " "
          (shell-quote host)
          " "
          (number->string port)
          " < "
          (shell-quote request)
          " > "
          (shell-quote response)
          " 2>/dev/null; else timeout "
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
          (shell-quote response)
          "; fi")))
    (run-unchecked command verbose?)))

(define (send-ack host port ack-file verbose?)
  (let ((command
         (string-append
          "timeout 2 nc -u -w 1 "
          (shell-quote host)
          " "
          (number->string port)
          " < "
          (shell-quote ack-file)
          " > /dev/null")))
    (run-unchecked command verbose?)))

(define-values (host port sequence attempts wait-seconds color? read-only? legacy-request? payload-only? parts)
  (parse (command-line-tail)
         (env "PSOC_NET_REPL_HOST")
         default-port
         default-sequence
         default-attempts
         default-wait-seconds
         #f
         #f
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
(define ack-file (repo-path ack-path))

(define request-frame-header-bytes (if legacy-request? 8 12))
(define request-checksum (and (not legacy-request?) (checksum-bytes (ascii-byte-list form))))
(define verbose? (not payload-only?))

(write-request request-file sequence form legacy-request? read-only? verbose?)
(when verbose?
  (say (string-append "request.bytes="
                      (number->string (+ request-frame-header-bytes (string-length form)))))
  (say (string-append "request.format=" (request-magic legacy-request? read-only?)))
  (when request-checksum
    (say (string-append "request.checksum=#x" (u32-hex request-checksum))))
  (say (string-append "sequence=" (number->string sequence))))

(let loop ((attempt 1))
  (send-attempt host port wait-seconds request-file response-file verbose?)
  (let ((response (read-bytes response-file)))
    (cond
      ((valid-response? response sequence)
       (let ((payload (take-after-header response)))
         (if payload-only?
             (display-payload-raw payload color?)
             (begin
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
               (display-payload-line payload color?)))
         (let ((ack-checksum (response-payload-checksum response)))
           (write-ack ack-file sequence ack-checksum verbose?)
           (when verbose?
             (say "ack.format=LPS4")
             (say (string-append "ack.response-checksum=#x" (u32-hex ack-checksum))))
           (send-ack host port ack-file verbose?))
         (exit 0)))
      ((>= attempt attempts)
       (if (null? response)
           (die "no response")
           (die (string-append "invalid response: " (bytes-hex response)))))
      (else
       (loop (+ attempt 1))))))
