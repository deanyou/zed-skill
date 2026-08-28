; SKILL sample for grammar test
/* block comment
   spanning lines */
(defun myFunc (a b @rest args "docstring")
  (let ((x (+ a b))
        (y 3.14))
    (when (> x y)
      (printf "x=%d y=%f hex=%x\n" x y 0xFF)
      t)))

(setq myVar '(1 2 . 3))
(setq pair '(a . b))

(foreach item '(1 2 3)
  (println item))

(printf "%s\n" "escape: \"quoted\"\ttab")
(setq ch ?a)
(setq nl ?\n)
(myFunc 1 2 ?key "value" other)
(let ((q `(a ,b ,@c)))
  q)
(setq neg -5 pos +2.5e3)
(defun %localVar ()
  (car '(x y)))
(setq arr [1 2 [3 4]])
(setq m {a 1 b {c 2}})
