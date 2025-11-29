; HTTP Request Syntax Highlighting for Zed
; Uses only valid nodes from the tree-sitter-http grammar

; HTTP Methods (GET, POST, PUT, DELETE, etc.)
(method) @keyword

; URLs/targets
(target) @string.special

; HTTP Version (HTTP/1.1, HTTP/2)
(http_version) @constant

; Request Separator (###)
(request_separator) @punctuation.special

; Headers
(header_name) @property
(header_value) @string

; Comments
(comment) @comment

; Request Body content
(body_content) @string

; Punctuation
":" @punctuation.delimiter
