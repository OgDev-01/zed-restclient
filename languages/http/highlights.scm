; HTTP Request Syntax Highlighting
; Simple pattern-based highlighting (full Tree-sitter grammar comes in Phase 4)

; HTTP Methods - highlight as keywords
; GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD, TRACE, CONNECT
(
  (identifier) @keyword.method
  (#match? @keyword.method "^(GET|POST|PUT|DELETE|PATCH|OPTIONS|HEAD|TRACE|CONNECT)$")
)

; URLs - highlight as special strings
; Matches http://, https://, and other URL patterns
(
  (string) @string.special.url
  (#match? @string.special.url "^https?://")
)

; Request delimiter ### - highlight as delimiter
(
  (operator) @keyword.delimiter
  (#match? @keyword.delimiter "^###")
)

; Headers - property style
; Matches "Header-Name:" pattern
(
  (property) @property
  (#match? @property "^[A-Za-z-]+:")
)

; Header values - strings
(string) @string

; Variables {{variable}} - highlight as parameters
; Matches {{variableName}} or {{$systemVar}}
(
  (variable) @variable.parameter
  (#match? @variable.parameter "^\\{\\{.*\\}\\}$")
)

; Comments - both # and // style
(comment) @comment

; HTTP version - HTTP/1.1, HTTP/2, etc.
(
  (identifier) @constant
  (#match? @constant "^HTTP/[0-9.]+$")
)

; Status codes in responses (if viewing response files)
(
  (number) @number
  (#match? @number "^[1-5][0-9]{2}$")
)

; JSON body content type
(
  (string) @string.special
  (#match? @string.special "application/json")
)

; Content-Type and other important headers
(
  (property) @property.special
  (#match? @property.special "^(Content-Type|Authorization|Accept|User-Agent):")
)

; Boolean values in JSON bodies
(
  (boolean) @constant.builtin.boolean
)

; Null values in JSON bodies
(
  (null) @constant.builtin
)

; Numbers in request bodies
(number) @number
