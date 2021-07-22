use reg_machine::{
    machine::{procedure::Procedure, value::TryFromValue},
    make_machine, math,
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

fn procedures() -> Vec<Procedure> {
    let mut procedures: Vec<Procedure> = vec![];
    procedures.push(Procedure::new("=", 2, math::equal));
    procedures.push(Procedure::new("rem", 2, |args| {
        let dividend = f64::try_from(&args[0]).unwrap();
        let divisor = f64::try_from(&args[1]).unwrap();
        dividend % divisor
    }));
    procedures
}

fn main() {
    let register_names = vec!["a", "b", "t"];
    let procedures = procedures();
    let mut machine = make_machine(register_names, &procedures, CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
