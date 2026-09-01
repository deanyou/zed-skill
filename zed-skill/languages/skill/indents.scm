; Indentation rules for Cadence SKILL (Lisp-style bracket indentation)

[
  "("
  "["
  "{"
] @indent.begin

[
  ")"
  "]"
  "}"
] @indent.end

; Closing brackets at the start of a line dedent
[
  ")"
  "]"
  "}"
] @outdent
