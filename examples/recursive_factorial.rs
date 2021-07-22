use reg_machine::{machine::procedure::Procedure, make_machine, math};

const CONTROLLER_TEXT: &str = r#"
(controller
   (perform (op print) (const "Please enter a number to compute the factorial:"))
   (assign n (op read))
   (assign continue (label fact-done))    ; set up final return address
 fact-loop
   (test (op =) (reg n) (const 1))
   (branch (label base-case))
   ;; Set up for the recursive call by saving n and continue.
   ;; Set up continue so that the computation will continue
   ;; at after-fact when the subroutine returns.
   (save continue)
   (save n)
   (assign n (op -) (reg n) (const 1))
   (assign continue (label after-fact))
   (goto (label fact-loop))
 after-fact
   (restore n)
   (restore continue)
   (assign val (op *) (reg n) (reg val))  ; val now contains n(n - 1)!
   (goto (reg continue))                  ; return to caller
 base-case
   (assign val (const 1))                 ; base case: 1! = 1
   (goto (reg continue))                  ; return to caller
 fact-done
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
    let register_names = vec!["continue", "n", "val"];
    let procedures = procedures();
    let mut machine = make_machine(register_names, &procedures, CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
