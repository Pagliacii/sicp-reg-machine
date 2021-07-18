use std::collections::HashMap;
use std::sync::Mutex;

use reg_machine::machine::value::Value;

pub struct Environment(Mutex<HashMap<String, Value>>);

impl Clone for Environment {
    fn clone(&self) -> Self {
        let mut environment: HashMap<String, Value> = HashMap::new();
        let base = self.0.lock().unwrap().clone();
        environment.extend(base);
        Self(Mutex::new(environment))
    }
}

impl Environment {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn lookup(&self, args: &Vec<Value>) -> Value {
        if args.len() < 1 {
            panic!("[LOOKUP] Missing a variable name.");
        }
        let var = args[0].to_string();
        let env = self.0.lock().unwrap();
        match env.get(&var) {
            Some(val) => val.clone(),
            None => panic!("Unbound variable {}", var),
        }
    }

    pub fn insert(&self, args: &Vec<Value>) {
        if args.len() < 2 {
            panic!("[DEFINE] Missing a value.");
        }
        let var = args[0].to_string();
        let val = args[1].clone();
        self.insert_value(var, val);
    }

    pub fn insert_value(&self, var: String, val: Value) {
        let mut env = self.0.lock().unwrap();
        env.insert(var, val);
    }

    pub fn update(&self, args: &Vec<Value>) {
        if args.len() < 2 {
            panic!("[DEFINE] Missing a value.");
        }
        let var = args[0].to_string();
        let mut env = self.0.lock().unwrap();
        match env.get_mut(&var) {
            Some(val) => *val = args[1].clone(),
            None => panic!("Unbound variable: SET! {}", var),
        }
    }

    pub fn extend(&self, args: &Vec<Value>) -> Self {
        if args.len() < 2 {
            panic!("[EXTEND] Missing values.");
        }
        let env = self.clone();
        let variables = &args[0];
        let values = &args[1];
        if let (Value::List(vars), Value::List(vals)) = (variables, values) {
            if vars.len() < vals.len() {
                panic!(
                    "Too many arguments supplied, vars = {} and vals = {}",
                    variables, values
                );
            } else if vars.len() > vals.len() {
                panic!(
                    "Too few arguments supplied, vars = {} and vals = {}",
                    variables, values
                );
            }
            env.extend_inner_map(vars, vals);
            env
        } else {
            panic!("[EXTEND] Unknown arguments: {} and {}", variables, values);
        }
    }

    fn extend_inner_map(&self, vars: &Vec<Value>, vals: &Vec<Value>) {
        let mut env = self.0.lock().unwrap();
        env.extend(
            vars.iter()
                .zip(vals.iter())
                .map(|(var, val)| (var.to_string(), val.clone())),
        );
    }
}
