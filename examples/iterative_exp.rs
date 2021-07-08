use std::collections::HashMap;

use reg_machine::{
    machine::{operation::Operation, Operations},
    make_machine,
};

const CONTROLLER_TEXT: &str = r#"
(controller
   (perform (op print) (const "[1] Please enter the base of exponentiation:"))
   (assign b (op read))
   (perform (op print) (const "[2] Please enter the exponent of exponentiation:"))
   (assign n (op read))
   (assign p (const 1))
 expt-iter
   (test (op =) (reg n) (const 0))
   (branch (label expt-done))
   (assign n (op -) (reg n) (const 1))
   (assign p (op *) (reg b) (reg p))
   (goto (label expt-iter))
 expt-done
   (perform (op print) (const "[3] Exponentiation:"))
   (perform (op print) (reg p))
 done)
"#;

fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert("=", Operation::new(|a: u64, b: u64| a == b));
    operations.insert("-", Operation::new(|a: u64, b: u64| a - b));
    operations.insert("*", Operation::new(|a: u64, b: u64| a * b));
    operations
}

fn main() {
    let register_names = vec!["b", "n", "p"];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
