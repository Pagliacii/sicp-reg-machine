use reg_machine::{
    machine::{procedure::Procedure, value::TryFromValue},
    make_machine, math,
};

const CONTROLLER_TEXT: &str = r#"
(controller
   (perform (op print) (const "Please enter a positive number:"))
   (assign x (op read))
   (assign g (const 1.0))
 test-g                                   ;; (sqrt-iter guess x)
   ;;; (good-enough? guess x)
   (assign t (op *) (reg g) (reg g))      ;; (square guess)
   (assign t (op -) (reg t) (reg x))      ;; (- (square guess) x)
   (assign t (op abs) (reg t))            ;; (abs (- (square guess) x))
   (test (op <) (reg t) (const 0.001))    ;; (< (abs (- (square guess) x)))
   (branch (label sqrt-done))
   ;;; (improve guess x)
   (assign t (op /) (reg x) (reg g))      ;; (/ x guess)
   (assign t (op +) (reg g) (reg t))      ;; (+ guess (/ x guess))
   (assign g (op /) (reg t) (const 2.0))  ;; (/ (+ guess (/ x guess)) 2.0)
   (goto (label test-g))                  ;; (sqrt-iter (improve guess x) x)
 sqrt-done
   (perform (op print) (reg g))
 done)
"#;

fn procedures() -> Vec<Procedure> {
    let mut procedures: Vec<Procedure> = vec![];
    procedures.push(Procedure::new("+", 2, math::addition));
    procedures.push(Procedure::new("-", 2, math::subtraction));
    procedures.push(Procedure::new("*", 2, math::multiplication));
    procedures.push(Procedure::new("/", 2, math::division));
    procedures.push(Procedure::new("<", 2, math::less_than));
    procedures.push(Procedure::new("abs", 1, |args| {
        let x = f64::try_from(&args[0]).unwrap();
        x.abs()
    }));
    procedures
}

fn main() {
    let register_names = vec!["g", "t", "x"];
    let procedures = procedures();
    let mut machine = make_machine(register_names, &procedures, &CONTROLLER_TEXT).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}
