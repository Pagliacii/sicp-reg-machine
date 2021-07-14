use std::collections::HashMap;

use reg_machine::{
    machine::{operation::Operation, Operations},
    make_machine,
};

const CONTROLLER_TEXT: &str = r#"
(controller
   (perform (op newline))
   (perform (op print) (const "Please enter a number or 'q' for quit: "))
   (assign n (op read))
   (test (op eq?) (reg n) (const q))
   (branch (label done))
   (test (op noninteger?) (reg n))
   (branch (label controller))
   (assign continue (label fib-done))
 fib-loop
   (test (op <) (reg n) (const 2))
   (branch (label immediate-answer))
   ;; set up to compute Fib(n-1)
   (save continue)
   (assign continue (label afterfib-n-1))
   (save n)                 ; save old value of n
   (assign n (op -) (reg n) (const 1))    ; clobber n to n-1
   (goto (label fib-loop))  ; perform recursive call
 afterfib-n-1     ; upon return, val contains Fib(n-1)
   (restore n)
   (restore continue)
   ;; set up to compute Fib(n-2)
   (assign n (op -) (reg n) (const 2))
   (save continue)
   (assign continue (label afterfib-n-2))
   (save val)               ; save Fib(n-1)
   (goto (label fib-loop))
 afterfib-n-2     ; upon return, val contains Fib(n-2)
   (assign n (reg val))     ; n now contains Fib(n-2)
   (restore val)            ; val now contains Fib(n-1)
   (restore continue)
   (assign val              ; Fib(n-1) + Fib(n-2)
           (op +) (reg val) (reg n))
   (goto (reg continue))    ; return to caller, answer is in val
 immediate-answer
   (assign val (reg n))     ; base case: Fib(n) = n
   (goto (reg continue))
 fib-done
   (perform (op print-stack-statistics))
   (perform (op print) (reg val))
   (perform (op initialize-stack))
   (goto (label controller))
 done)
"#;

fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert("newline", Operation::new(|| println!()));
    operations.insert("eq?", Operation::new(|a: String, b: String| a == b));
    operations.insert(
        "noninteger?",
        Operation::new(|s: String| s.parse::<i32>().is_err()),
    );
    operations.insert("<", Operation::new(|a: i32, b: i32| a < b));
    operations.insert("+", Operation::new(|a: i32, b: i32| a + b));
    operations.insert("-", Operation::new(|a: i32, b: i32| a - b));
    operations
}

fn main() {
    let register_names = vec!["continue", "n", "val"];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
