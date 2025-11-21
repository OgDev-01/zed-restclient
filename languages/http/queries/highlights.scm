; HTTP Request Syntax Highlighting
; Tree-sitter highlight queries for HTTP request files

; ============================================================================
; HTTP Methods - highlight as keywords
; ============================================================================
(method) @keyword.method

; ============================================================================
; URLs - highlight as special strings
; ============================================================================
(target) @string.special.url

; ============================================================================
; HTTP Version
; ============================================================================
(http_version) @constant

; ============================================================================
; Request Separator - highlight as delimiter
; ============================================================================
(request_separator) @keyword.delimiter

"###" @keyword.delimiter

; ============================================================================
; Headers
; ============================================================================
(header_name) @property

(header_value) @string

; Special headers - highlight differently
((header_name) @property.special
  (#match? @property.special "^(Content-Type|Authorization|Accept|User-Agent|Accept-Encoding|Cache-Control|Connection|Host|Origin|Referer)$"))

; ============================================================================
; Variables - {{variable}} (regex-based highlighting in URLs and headers)
; ============================================================================
; Variables are matched as part of target URLs and header values
; Additional regex-based highlighting can be added in the language config

; ============================================================================
; Comments
; ============================================================================
(comment) @comment

"#" @comment
"//" @comment

; ============================================================================
; Request Body
; ============================================================================
(body_content) @string

; ============================================================================
; Punctuation
; ============================================================================
":" @punctuation.delimiter

; ============================================================================
; Content-Type values - highlight JSON, XML, GraphQL
; ============================================================================
((header_value) @string.special
  (#match? @string.special "application/(json|xml|graphql|x-www-form-urlencoded)"))

((header_value) @string.special
  (#match? @string.special "text/(xml|html|plain|css|javascript)"))

((header_value) @string.special
  (#match? @string.special "multipart/form-data"))

; ============================================================================
; Common authentication patterns
; ============================================================================

; Bearer tokens in Authorization header
((header_value) @string.special
  (#match? @string.special "^Bearer "))

; Basic auth
((header_value) @string.special
  (#match? @string.special "^Basic "))

; API keys
((header_value) @string.special
  (#match? @string.special "^(API-Key|X-API-Key)"))

; ============================================================================
; HTTP Status Codes (if present in header values)
; ============================================================================
((header_value) @number
  (#match? @number "^[1-5][0-9]{2}$"))

; ============================================================================
; Common HTTP methods in uppercase
; ============================================================================
((method) @keyword.method.get
  (#eq? @keyword.method.get "GET"))

((method) @keyword.method.post
  (#eq? @keyword.method.post "POST"))

((method) @keyword.method.put
  (#eq? @keyword.method.put "PUT"))

((method) @keyword.method.delete
  (#eq? @keyword.method.delete "DELETE"))

((method) @keyword.method.patch
  (#eq? @keyword.method.patch "PATCH"))
