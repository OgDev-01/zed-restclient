; HTTP Request Syntax Highlighting
;
; This file is maintained for backward compatibility with older Tree-sitter setups.
; The actual highlight queries are defined in queries/highlights.scm
;
; For the complete and up-to-date syntax highlighting rules, see:
; - queries/highlights.scm (main highlighting rules)
; - queries/injections.scm (embedded language support for JSON, XML, GraphQL)

; Include the main highlight queries
; Note: This is a simple redirect - the actual queries are in the queries/ directory

; HTTP Methods - highlight as keywords
(method) @keyword.method

; URLs - highlight as special strings
(target) @string.special.url

; HTTP Version
(http_version) @constant

; Request Separator
(request_separator) @keyword.delimiter

; Headers
(header_name) @property
(header_value) @string

; Special headers
((header_name) @property.special
  (#match? @property.special "^(Content-Type|Authorization|Accept|User-Agent|Accept-Encoding|Cache-Control|Connection|Host|Origin|Referer)$"))

; Comments
(comment) @comment

; Request Body
(body_content) @string

; Punctuation
":" @punctuation.delimiter

; Content-Type values
((header_value) @string.special
  (#match? @string.special "application/(json|xml|graphql|x-www-form-urlencoded)"))

((header_value) @string.special
  (#match? @string.special "text/(xml|html|plain|css|javascript)"))

((header_value) @string.special
  (#match? @string.special "multipart/form-data"))

; Authentication patterns
((header_value) @string.special
  (#match? @string.special "^Bearer "))

((header_value) @string.special
  (#match? @string.special "^Basic "))
