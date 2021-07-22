use reg_machine::{machine::procedure::Procedure, make_machine, math};

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

fn procedures() -> Vec<Procedure> {
    let mut procedures: Vec<Procedure> = vec![];
    procedures.push(Procedure::new("=", 2, math::equal));
    procedures.push(Procedure::new("-", 2, math::subtraction));
    procedures.push(Procedure::new("*", 2, math::multiplication));
    procedures
}

fn main() {
    let register_names = vec!["b", "n", "p"];
    let procedures = procedures();
    let mut machine = make_machine(register_names, &procedures, CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
