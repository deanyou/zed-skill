; Outline / symbol list for Cadence SKILL (Zed captures: @item, @context, @name)

((list
   .
   (symbol) @context
   (symbol) @name) @item
 (#any-of? @context "defun" "defmacro" "procedure"))

; Top-level variable definitions
((list
   .
   (symbol) @context
   (symbol) @name) @item
 (#any-of? @context "setq" "setf"))
