use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use reg_machine::machine::{
    procedure::Procedure,
    value::{ToValue, Value},
};

use super::primitive::primitive_procedures;

struct Environment(Mutex<HashMap<String, Value>>);

impl Clone for Environment {
    fn clone(&self) -> Self {
        let mut environment: HashMap<String, Value> = HashMap::new();
        let base = self.0.lock().unwrap().clone();
        environment.extend(base);
        Self(Mutex::new(environment))
    }
}

impl Environment {
    fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    fn lookup(&self, args: &[Value]) -> Value {
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

    fn insert(&self, args: &[Value]) {
        if args.len() < 2 {
            panic!("[DEFINE] Missing a value.");
        }
        let var = args[0].to_string();
        let val = args[1].clone();
        self.insert_value(var, val);
    }

    fn insert_value(&self, var: String, val: Value) {
        let mut env = self.0.lock().unwrap();
        env.insert(var, val);
    }

    fn update(&self, args: &[Value]) {
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

    fn extend(&self, args: &[Value]) -> Self {
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

lazy_static! {
    static ref PRIMITIVE_PROCEDURES: Vec<Procedure> = primitive_procedures();
    static ref ENVIRONMENTS: Mutex<Vec<Environment>> = {
        let global_env: Environment = Environment::new();
        for proc in PRIMITIVE_PROCEDURES.iter() {
            global_env.insert_value(
                proc.get_name(),
                vec![Value::new("primitive"), proc.clone().to_value()].to_value(),
            );
        }
        global_env.insert_value("true".into(), Value::Boolean(true));
        global_env.insert_value("false".into(), Value::Boolean(false));
        Mutex::new(vec![global_env])
    };
}

pub fn get_global_environment() -> Value {
    let mut envs = ENVIRONMENTS.lock().unwrap();
    while envs.len() > 1 {
        // drop other environments except the global one.
        envs.pop();
    }
    Value::Pointer(0)
}

pub fn manipulate_env(op: &'static str, env: &Value, args: &[Value]) -> Value {
    let mut envs = ENVIRONMENTS.lock().unwrap();
    let env_ptr = if let Value::Pointer(p) = env {
        if *p >= envs.len() {
            panic!("Unknown environment: {}", p);
        }
        *p
    } else {
        panic!("Unknown environment: {}", env);
    };
    match op {
        "lookup" => envs[env_ptr].lookup(args),
        "define" => {
            envs[env_ptr].insert(args);
            Value::Pointer(env_ptr)
        }
        "update" => {
            envs[env_ptr].update(args);
            Value::Pointer(env_ptr)
        }
        "extend" => {
            let new_ptr: usize;
            if env_ptr == 0 {
                // extend the global environment
                let env = envs[0].extend(args);
                envs.push(env);
                new_ptr = envs.len() - 1;
            } else if env_ptr == envs.len() - 1 {
                // extend the last one
                let env = envs.last().unwrap().extend(args);
                envs.push(env);
                new_ptr = env_ptr + 1;
            } else {
                // extend an existed environment
                let env = envs[env_ptr].extend(args);
                envs[env_ptr + 1] = env;
                new_ptr = env_ptr + 1;
            }
            Value::Pointer(new_ptr)
        }
        other => panic!("[Environment] Unknown request: {}", other),
    }
}

#[cfg(test)]
mod environment_tests {
    use super::*;
    use reg_machine::machine::value::TryFromValue;

    #[test]
    fn test_extend_environment() {
        let vars = Value::new(vec![Value::new("a"), Value::new("b"), Value::new("c")]);
        let vals = Value::new(vec![Value::new(1), Value::new(1.0), Value::new(1u64)]);
        let env = usize::try_from(get_global_environment()).unwrap();
        let env = manipulate_env("extend", env, &vec![vars, vals]);
        let env = usize::try_from(env).unwrap();
        assert_eq!(
            Value::Num(1.0),
            manipulate_env("lookup", env, &vec![Value::new("a")])
        );
        assert_eq!(
            Value::Num(1.0),
            manipulate_env("lookup", env, &vec![Value::new("b")])
        );
        assert_eq!(
            Value::Num(1.0),
            manipulate_env("lookup", env, &vec![Value::new("c")])
        );
    }

    #[test]
    fn test_define_variable() {
        let env = usize::try_from(get_global_environment()).unwrap();
        manipulate_env("define", env, &vec![Value::new("a"), Value::new(1)]);
        assert_eq!(
            Value::Num(1.0),
            manipulate_env("lookup", env, &vec![Value::new("a")])
        );
    }

    #[test]
    fn test_set_variable_value() {
        let env = usize::try_from(get_global_environment()).unwrap();
        manipulate_env("define", env, &vec![Value::new("a"), Value::new(1)]);
        manipulate_env("update", env, &vec![Value::new("a"), Value::new(2)]);
        assert_eq!(
            Value::Num(2.0),
            manipulate_env("lookup", env, &vec![Value::new("a")])
        );
    }

    #[test]
    fn test_lookup_variable_value() {
        let env = usize::try_from(get_global_environment()).unwrap();
        manipulate_env("define", env, &vec![Value::new("a"), Value::new(1)]);
        let val = manipulate_env("lookup", env, &vec![Value::new("a")]);
        assert_eq!(Value::new(1), val);
        manipulate_env("update", env, &vec![Value::new("a"), Value::new(2)]);
        let val = manipulate_env("lookup", env, &vec![Value::new("a")]);
        assert_eq!(Value::new(2), val);
    }
}
