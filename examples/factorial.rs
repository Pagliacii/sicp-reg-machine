use reg_machine::{
    machine::{procedure::Procedure, value::TryFromValue},
    make_machine, math,
};

const CONTROLLER_TEXT: &str = r#"
(controller
   (assign p (const 1))
   (assign c (const 1))
 test-c
   (test (op >) (reg c) (reg n))
   (branch (label factorial-done))
   (assign p (op *) (reg p) (reg c))
   (assign c (op +) (reg c) (const 1))
   (goto (label test-c))
 factorial-done)
"#;

fn procedures() -> Vec<Procedure> {
    let mut procedures: Vec<Procedure> = vec![];
    procedures.push(Procedure::new(">", 2, math::greater_than));
    procedures.push(Procedure::new("*", 2, math::multiplication));
    procedures.push(Procedure::new("+", 2, math::addition));
    procedures
}

fn main() {
    let register_names = vec!["n", "p", "c"];
    let mut machine = make_machine(register_names, &procedures(), CONTROLLER_TEXT).unwrap();
    machine.set_register_content("n", 16).unwrap();
    assert_eq!(Ok("Done"), machine.start());
    let value = machine.get_register_content("p").unwrap();
    println!("factorial(16) = {}", u64::try_from(&value).unwrap());
}
