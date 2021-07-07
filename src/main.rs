use std::collections::HashMap;
use std::fs;
use std::io;

use reg_machine::{
    machine::{operation::Operation, Operations},
    make_machine,
};

fn read_line_buffer() -> String {
    // Read one line of input buffer-style
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

fn main() {
    let mut operations: Operations = HashMap::new();
    operations.insert("print", Operation::new(|s: String| println!("\n{}", s)));
    operations.insert("read", Operation::new(read_line_buffer));
    operations.insert("eq?", Operation::new(|a: String, b: String| a == b));
    operations.insert(
        "noninteger?",
        Operation::new(|s: String| s.parse::<i32>().is_err()),
    );
    operations.insert("<", Operation::new(|a: i32, b: i32| a < b));
    operations.insert("+", Operation::new(|a: i32, b: i32| a + b));
    operations.insert("-", Operation::new(|a: i32, b: i32| a - b));

    let controller_text = fs::read_to_string("./tests/rml_insts.scm").unwrap();
    let mut machine =
        make_machine(vec!["continue", "n", "val"], &operations, &controller_text).unwrap();
    println!("{}", machine.start().unwrap());
}
