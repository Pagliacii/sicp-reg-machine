use reg_machine::{
    machine::{procedure::Procedure, value::TryFromValue},
    make_machine,
};

const CONTROLLER_TEXT: &str = r#"
(controller
   (perform (op print) (const "Please enter a positive number:"))
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

fn procedures() -> Vec<Procedure> {
    let mut procedures: Vec<Procedure> = vec![];
    procedures.push(Procedure::new("good-enough?", 2, |args| {
        let guess = f64::try_from(&args[0]).unwrap();
        let x = f64::try_from(&args[1]).unwrap();
        0.001 > (guess.powi(2) - x).abs()
    }));
    procedures.push(Procedure::new("improve", 2, |args| {
        let guess = f64::try_from(&args[0]).unwrap();
        let x = f64::try_from(&args[1]).unwrap();
        (guess + x / guess) / 2.0
    }));
    procedures
}

fn main() {
    let register_names = vec!["g", "t", "x"];
    let procedures = procedures();
    let mut machine = make_machine(register_names, &procedures, CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
