use std::collections::HashMap;

use reg_machine::{
    machine::{operation::Operation, Operations},
    make_machine,
};

const CONTROLLER_TEXT: &str = r#"
(controller
   (assign x (op read))
   (assign g (const 1.0))
 test-g
   (test (op good-enough?) (reg g) (reg x))
   (branch (label sqrt-done))
   (assign t (op improve) (reg g) (reg x))
   (assign g (reg t))
   (goto (label test-g))
 sqrt-done
   (perform (op print) (reg g))
 done)
"#;

fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert(
        "good-enough?",
        Operation::new(|guess: f64, x: f64| 0.001 > (guess.powi(2) - x).abs()),
    );
    operations.insert(
        "improve",
        Operation::new(|guess: f64, x: f64| (guess + x / guess) / 2.0),
    );
    operations
}

fn main() {
    let register_names = vec!["g", "t", "x"];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
