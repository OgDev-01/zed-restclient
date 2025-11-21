; HTTP Request Body Injection Queries
; Tree-sitter injection queries for embedded languages in HTTP request bodies

; ============================================================================
; JSON Injection - Inject JSON grammar when Content-Type is application/json
; ============================================================================
((request
  (headers
    (header
      (header_name) @_name
      (header_value) @_value))
  (body
    (body_content) @injection.content))
  (#eq? @_name "Content-Type")
  (#match? @_value "application/json")
  (#set! injection.language "json"))

; ============================================================================
; XML Injection - Inject XML grammar when Content-Type is application/xml
; ============================================================================
((request
  (headers
    (header
      (header_name) @_name
      (header_value) @_value))
  (body
    (body_content) @injection.content))
  (#eq? @_name "Content-Type")
  (#match? @_value "(application|text)/xml")
  (#set! injection.language "xml"))

; ============================================================================
; GraphQL Injection - Inject GraphQL grammar when Content-Type is application/graphql
; ============================================================================
((request
  (headers
    (header
      (header_name) @_name
      (header_value) @_value))
  (body
    (body_content) @injection.content))
  (#eq? @_name "Content-Type")
  (#match? @_value "application/graphql")
  (#set! injection.language "graphql"))

; ============================================================================
; HTML Injection - Inject HTML grammar when Content-Type is text/html
; ============================================================================
((request
  (headers
    (header
      (header_name) @_name
      (header_value) @_value))
  (body
    (body_content) @injection.content))
  (#eq? @_name "Content-Type")
  (#match? @_value "text/html")
  (#set! injection.language "html"))

; ============================================================================
; JavaScript Injection - Inject JavaScript grammar when Content-Type is text/javascript
; ============================================================================
((request
  (headers
    (header
      (header_name) @_name
      (header_value) @_value))
  (body
    (body_content) @injection.content))
  (#eq? @_name "Content-Type")
  (#match? @_value "(text|application)/javascript")
  (#set! injection.language "javascript"))

; ============================================================================
; Heuristic-based JSON injection (when body starts with { or [)
; ============================================================================
((body
  (body_content) @injection.content)
  (#match? @injection.content "^\\s*[{\\[]")
  (#set! injection.language "json"))

; ============================================================================
; Heuristic-based XML injection (when body starts with <)
; ============================================================================
((body
  (body_content) @injection.content)
  (#match? @injection.content "^\\s*<")
  (#set! injection.language "xml"))

; ============================================================================
; Heuristic-based GraphQL injection (when body starts with query/mutation)
; ============================================================================
((body
  (body_content) @injection.content)
  (#match? @injection.content "^\\s*(query|mutation|subscription)")
  (#set! injection.language "graphql"))
