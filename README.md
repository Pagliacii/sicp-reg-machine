# Register Machine

The machine details could be found in the SICP chapter 5. See [here](https://sarabander.github.io/sicp/html/5_002e1.xhtml#g_t5_002e1).

## Exercise 5.51

> Develop a rudimentary implementation of Scheme in C (or some other low-level language of your choice) by translating the explicit-control evaluator of [Section 5.4](https://sarabander.github.io/sicp/html/5_002e4.xhtml#g_t5_002e4) into C. In order to run this code you will need to also provide appropriate storage-allocation routines and other run-time support.

## Instruction Summary

```scheme
; access the register contents
(reg <register-name>)
; a constant value
(const <constant-value>)
; a control label
(label <label-name>)
; test a condition and jump to the control label
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

## Machines

| Machines  | Details                                                                                                                                                                            | Code                                 |
| --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| Fibonacci | See [Section 5.1.4](https://sarabander.github.io/sicp/html/5_002e1.xhtml#g_t5_002e1_002e4) and [Figure 5.12](https://sarabander.github.io/sicp/html/5_002e1.xhtml#Figure-5_002e12) | [fibonacci.rs](src/bin/fibonacci.rs) |
| GCD V1    | See [Section 5.1.1](https://sarabander.github.io/sicp/html/5_002e1.xhtml#g_t5_002e1_002e1)                                                                                         | [gcd.rs](src/bin/gcd.rs)             |

## Running

```shell
$ git clone https://github.com/Pagliacii/sicp-reg-machine
$ cd sicp-reg-machine
# List all machines
$ ls examples
# Running machine
$ cargo run --example <machine-name>
```

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
