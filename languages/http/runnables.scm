; Runnables for HTTP requests
; This shows a run button (⚡) next to each HTTP request in the gutter
;
; The @run capture marks where the run button appears
; Other captures become environment variables with ZED_CUSTOM_ prefix

; Match HTTP requests and show run button on the method line
; The request node contains the full request (method, headers, body)
(request
  (method_line
    (method) @run @_method
    (target) @_url))

; Alternative: simpler pattern that just matches the method
; (method) @run
