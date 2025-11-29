; HTTP Request Syntax Highlighting for Zed
; Uses standard Zed theme captures for consistent coloring

; HTTP Methods - highlight as keywords (GET, POST, PUT, DELETE, etc.)
(method) @keyword

; URLs - highlight as links
(target) @link_uri

; HTTP Version (HTTP/1.1, HTTP/2, etc.)
(http_version) @constant

; Request Separator (###)
(request_separator) @punctuation.special

; Headers
(header_name) @property
(header_value) @string

; Comments (# or //)
(comment) @comment

; Request Body - treat as embedded content
(body_content) @embedded

; Punctuation
":" @punctuation.delimiter

; Variables {{variable_name}} - highlight the whole variable
(variable) @variable.special

; Variable name inside braces
(variable_name) @variable.special

; Variable braces
"{{" @punctuation.bracket
"}}" @punctuation.bracket

; Special Content-Type values
((header_value) @string.special
  (#match? @string.special "application/(json|xml|graphql|x-www-form-urlencoded)"))

((header_value) @string.special
  (#match? @string.special "text/(xml|html|plain|css|javascript)"))

((header_value) @string.special
  (#match? @string.special "multipart/form-data"))

; Authentication tokens (Bearer, Basic)
((header_value) @string.special
  (#match? @string.special "^Bearer "))

((header_value) @string.special
  (#match? @string.special "^Basic "))
