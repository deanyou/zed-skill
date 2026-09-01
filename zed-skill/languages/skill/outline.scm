; Outline / symbol list for Cadence SKILL

((list
   .
   (symbol) @_def
   (symbol) @name) @item
 (#any-of? @_def "defun" "defmacro" "procedure")
 (#set! "kind" "function"))

; Top-level variable definitions
((list
   .
   (symbol) @_def
   (symbol) @name) @item
 (#any-of? @_def "setq" "defvar" "defglobal")
 (#set! "kind" "variable"))
