# Register Machine

The machine details could be found at the SICP chapter 5. See [here](https://sarabander.github.io/sicp/html/5_002e1.xhtml#g_t5_002e1).

## Instruction Summary

```scheme
; access the register contents
(reg <register-name>)
; a constant value
(const <constant-value>)
; a controll label
(label <label-name>)
; test a condition and jump to the controll label
(test (op <operation-name>) <input_1> ... <input_n>)
(branch (label <label-name>)) ; only jump if the preceded test passes
; go to label immediately
(goto (label <label-name>))
; or go to label holds in the register
(goto (reg <register-name>))
; perform an operation
(perform (op <operation-name>) <input_1> .. <input_n>)
; assignment
(assign <register-name> (reg <register-name>))
(assign <register-name> (const <constant-value>))
(assign <register-name> (op <operation-name>) <input_1> .. <input_n>)
(assign <register-name> (label <label-name>))
; instructions to use the stack
(save <register-name>)
(restore <register-name>)
```

Valid kinds of constant value:

- `(const 123)` is the number `123`,
- `(const "abc")` is the string `"abc"`,
- `(const abc)` is the symbol `abc`,
- `(const (a b c))` is the list `(a b c)`,
- and `(const ())` is the empty list.
