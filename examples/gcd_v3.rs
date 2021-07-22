use reg_machine::{
    machine::{procedure::Procedure, value::TryFromValue},
    make_machine, math,
};

const CONTROLLER_TEXT: &str = r#"
(controller
 test-b
   (test (op =) (reg b) (const 0))
   (branch (label gcd-done))
   (assign t (reg a))
 rem-loop
   (test (op <) (reg t) (reg b))
   (branch (label rem-done))
   (assign t (op -) (reg t) (reg b))
   (goto (label rem-loop))
 rem-done
   (assign a (reg b))
   (assign b (reg t))
   (goto (label test-b))
 gcd-done)
"#;

fn procedures() -> Vec<Procedure> {
    let mut procedures: Vec<Procedure> = vec![];
    procedures.push(Procedure::new("=", 2, math::equal));
    procedures.push(Procedure::new("<", 2, math::less_than));
    procedures.push(Procedure::new("-", 2, math::subtraction));
    procedures
}

fn main() {
    let register_names = vec!["a", "b", "t"];
    let procedures = procedures();
    let mut machine = make_machine(register_names, &procedures, &CONTROLLER_TEXT).unwrap();
    machine.set_register_content("a", 1023).unwrap();
    machine.set_register_content("b", 27).unwrap();
    assert_eq!(Ok("Done"), machine.start());
    let value = machine.get_register_content("a").unwrap();
    println!("gcd(1023, 27) = {}", i32::try_from(&value).unwrap());
}
