; SKILL (Cadence Virtuoso) syntax highlighting
; Matches node names from tree-sitter-skill grammar:
;   comment block_comment string escape_sequence number
;   character keyword boolean symbol list quoting unquoting

; ---- Comments ----
(comment) @comment
(block_comment) @comment

; ---- Strings ----
(string) @string
(escape_sequence) @string.escape

; ---- Numbers ----
(number) @number

; ---- Character literals (?a ?\n) ----
(character) @constant

; ---- Keyword arguments (?key ?count) ----
((keyword) @variable.special)

; ---- Booleans (t / nil) ----
((boolean) @constant.builtin)

; ---- Quote / quasiquote / unquote markers ----
(quoting ["'" "`"] @operator)
(unquoting ["," (unquote_splicing)] @operator)

; ---- Symbols: base style ----
((symbol) @variable)

; ---- Function calls: first symbol of a list ----
((list . (symbol) @function.call))

; ---- SKILL special forms / macros (override function.call) ----
((list . (symbol) @keyword)
 (#any-of? @keyword
   "defun" "defmacro" "defstruct" "defGlobal" "defprop" "defMath"
   "let" "let*" "prog" "progn" "if" "if*" "when" "unless"
   "cond" "case" "caseq" "setq" "setf" "set"
   "foreach" "for" "while" "until" "loop" "do"
   "lambda" "quote" "quasiquote" "unquote" "unquote-splicing"
   "declare" "catch" "throw" "unwind-protect" "protect"
   "return" "returnFrom" "go" "error" "warn"
   "and" "or" "not" "null" "the"))

; ---- Common built-in functions ----
((symbol) @function.builtin
 (#any-of? @function.builtin
   "car" "cdr" "caar" "cadr" "cdar" "cddr" "caddr" "cadddr"
   "cons" "list" "append" "reverse" "length" "nth" "nthcdr" "last"
   "map" "mapc" "mapcar" "mapcan" "maplist" "mapcon" "mapinto"
   "eq" "equal" "neq" "nequal" "memq" "member" "assq" "assoc"
   "printf" "println" "print" "prin1" "sprintf" "fprintf" "fprintfn"
   "strcat" "strlen" "strncmp" "substring" "index" "rexMatchp"
   "get" "putprop" "getf" "plist" "getq" "setplist"
   "plus" "minus" "times" "quotient" "difference" "remainder"
   "min" "max" "abs" "round" "floor" "ceiling" "truncate" "fixp"
   "numberp" "stringp" "symbolp" "listp" "consp" "arrayp" "typep"
   "sort" "sortcar" "remove" "remq" "delete" "nconc"
   "load" "loadi" "eval" "apply" "funcall" "funobj"
   "dbOpenCellViewByType" "dbCreateInst" "dbGetQ" "dbSetq" "dbMakeNet"
   "geGetEditCellView" "geGetSelSet" "hiGetCurrentWindow" "leHiCreateInst"))
