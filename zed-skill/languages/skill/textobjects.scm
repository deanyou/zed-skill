; Text objects (expand selection) for Cadence SKILL

((list
   .
   (symbol) @_def) @function.around
 (#any-of? @_def "defun" "defmacro" "procedure"))

((list
   .
   (symbol) @_def) @function.inside
 (#any-of? @_def "defun" "defmacro" "procedure"))

(comment) @comment.inside
(comment) @comment.around
(block_comment) @comment.inside
(block_comment) @comment.around
