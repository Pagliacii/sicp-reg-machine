use std::collections::HashMap;

use reg_machine::{
    machine::{operation::Operation, Operations},
    make_machine,
};

const CONTROLLER_TEXT: &str = r#"
(controller
 gcd-loop
   (assign a (op read))
   (assign b (op read))
 test-b
   (test (op =)
         (reg b)
         (const 0))
   (branch (label gcd-done))
   (assign t
           (op rem)
           (reg a)
           (reg b))
   (assign a (reg b))
   (assign b (reg t))
   (goto (label test-b))
 gcd-done
   (perform (op print)
            (reg a))
   (goto (label gcd-loop)))
"#;

fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert("=", Operation::new(|a: i32, b: i32| a == b));
    operations.insert("rem", Operation::new(|a: i32, b: i32| a % b));
    operations
}

fn main() {
    let register_names = vec!["a", "b", "t"];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
