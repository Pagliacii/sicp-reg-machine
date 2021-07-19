use std::fs::read_to_string;

use reg_machine::make_machine;

mod operations;
mod supports;
use operations::operations;

fn init_logging(_log_path: &str) {
    env_logger::init();
}

fn main() {
    init_logging("examples/ec_evaluator/logs");
    let controller_text: String =
        read_to_string("examples/ec_evaluator/controller.scm").expect("Couldn't read from file");
    let register_names = vec![
        // `exp` is used to hold the expression to be evaluated
        "exp",
        // `env` contains the environment in which the evaluation is to be performed
        "env",
        // At the end of an evaluation, `val` contains the value obtained by
        // evaluating the expression in the designated environment
        "val",
        // The `continue` register is used to implement recursion,
        // as explained in Section 5.1.4.
        "continue",
        // The registers `proc`, `argl`, and `unev` are used in evaluating combinations.
        "proc", "argl", "unev",
    ];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &controller_text).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
