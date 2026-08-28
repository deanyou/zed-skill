; Bracket pair definitions for SKILL
; Powers matched-bracket highlighting and rainbow (per-level) bracket colors.
; Nesting colors come from the theme `accents` and require the
; `colorize_brackets` editor/language setting to be enabled.

("(" @open
  ")" @close)

("[" @open
  "]" @close)

("{" @open
  "}" @close)

; String quotes pair for bracket matching, but keep them out of rainbow colors
(("\"" @open
  "\"" @close)
  (#set! rainbow.exclude))
