use reg_machine::{machine::procedure::Procedure, make_machine, math};

const CONTROLLER_TEXT: &str = r#"
(controller
   (perform (op print) (const "[1] Please enter the base of exponentiation:"))
   (assign b (op read))
   (perform (op print) (const "[2] Please enter the exponent of exponentiation:"))
   (assign n (op read))
   (assign continue (label expt-done))    ; set up final return address
 expt-loop
   (test (op =) (reg n) (const 0))
   (branch (label base-case))
   ;; Set up for the recursive call by saving n and continue.
   ;; Set up continue so that the computation will continue
   ;; at after-expt when the subroutine returns.
   (save continue)
   (save n)
   (assign n (op -) (reg n) (const 1))
   (assign continue (label after-expt))
   (goto (label expt-loop))
 after-expt
   (restore n)
   (restore continue)
   (assign val (op *) (reg b) (reg val))  ; val now contains b * b^(n - 1)
   (goto (reg continue))                  ; return to caller
 base-case
   (assign val (const 1))                 ; base case: b^0 = 1
   (goto (reg continue))                  ; return to caller
 expt-done
   (perform (op print) (const "[3] Exponentiation:"))
   (perform (op print) (reg val))
 done)
"#;

fn procedures() -> Vec<Procedure> {
    let mut procedures: Vec<Procedure> = vec![];
    procedures.push(Procedure::new("=", 2, math::equal));
    procedures.push(Procedure::new("-", 2, math::subtraction));
    procedures.push(Procedure::new("*", 2, math::multiplication));
    procedures
}

fn main() {
    let register_names = vec!["b", "continue", "n", "val"];
    let procedures = procedures();
    let mut machine = make_machine(register_names, &procedures, CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
