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
