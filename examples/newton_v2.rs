use std::collections::HashMap;
use std::io;

use reg_machine::{
    machine::{operation::Operation, Operations},
    make_machine,
};

const CONTROLLER_TEXT: &str = r#"
(controller
   (assign x (op read))
   (assign g (const 1.0))
 test-g                                   ;; (sqrt-iter guess x)
   ;;; (good-enough? guess x)
   (assign t (op *) (reg g) (reg g))      ;; (square guess)
   (assign t (op -) (reg t) (reg x))      ;; (- (square guess) x)
   (assign t (op abs) (reg t))            ;; (abs (- (square guess) x))
   (test (op <) (reg t) (const 0.001))    ;; (< (abs (- (square guess) x)))
   (branch (label sqrt-done))
   ;;; (improve guess x)
   (assign t (op /) (reg x) (reg g))      ;; (/ x guess)
   (assign t (op +) (reg g) (reg t))      ;; (+ guess (/ x guess))
   (assign g (op /) (reg t) (const 2.0))  ;; (/ (+ guess (/ x guess)) 2.0)
   (goto (label test-g))                  ;; (sqrt-iter (improve guess x) x)
 sqrt-done
   (perform (op print) (reg g))
 done)
"#;

fn read_line_buffer() -> String {
    // Read one line of input buffer-style
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert("read", Operation::new(read_line_buffer));
    operations.insert("print", Operation::new(|a: f64| println!("{}", a)));
    operations.insert("+", Operation::new(|a: f64, b: f64| a + b));
    operations.insert("-", Operation::new(|a: f64, b: f64| a - b));
    operations.insert("*", Operation::new(|a: f64, b: f64| a * b));
    operations.insert("/", Operation::new(|a: f64, b: f64| a / b));
    operations.insert("<", Operation::new(|a: f64, b: f64| a < b));
    operations.insert("abs", Operation::new(|a: f64| a.abs()));
    operations
}

fn main() {
    let register_names = vec!["g", "t", "x"];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
