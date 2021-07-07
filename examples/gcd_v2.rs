use std::collections::HashMap;
use std::io;

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
    operations.insert("=", Operation::new(|b: i32| 0 == b));
    operations.insert("rem", Operation::new(|a: i32, b: i32| a % b));
    operations.insert("read", Operation::new(read_line_buffer));
    operations.insert("print", Operation::new(|a: i32| println!("{}", a)));
    operations
}

fn main() {
    let register_names = vec!["a", "b", "t"];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
