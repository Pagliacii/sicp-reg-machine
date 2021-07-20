use std::collections::HashMap;

use reg_machine::machine::{operation::Operation, value::Value, Operations};

use super::supports::{
    environment::{get_global_environment, manipulate_env},
    io::{announce_output, prompt_for_input, read, user_print},
    list::*,
    primitive::apply_primitive_procedure,
    syntax::*,
};

pub fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert("read", Operation::new(read));
    operations.insert("debug", Operation::new(|v: Value| println!("{:?}", v)));
    operations.insert("prompt-for-input", Operation::new(prompt_for_input));
    operations.insert("announce-output", Operation::new(announce_output));
    operations.insert("user-print", Operation::new(user_print));
    operations.insert(
        "get-global-environment",
        Operation::new(get_global_environment),
    );
    operations.insert(
        "lookup-variable-value",
        Operation::new(|var: Value, env_ptr: usize| manipulate_env("lookup", env_ptr, &vec![var])),
    );
    operations.insert(
        "set-variable-value!",
        Operation::new(|var: Value, val: Value, env_ptr: usize| {
            manipulate_env("update", env_ptr, &vec![var, val])
        }),
    );
    operations.insert(
        "extend-environment",
        Operation::new(|vars: Value, vals: Value, env_ptr: usize| {
            manipulate_env("extend", env_ptr, &vec![vars, vals])
        }),
    );
    operations.insert(
        "define-variable!",
        Operation::new(|var: Value, val: Value, env_ptr: usize| {
            manipulate_env("define", env_ptr, &vec![var, val]);
        }),
    );
    operations.insert("self-evaluating?", Operation::new(is_self_evaluating));
    operations.insert("variable?", Operation::new(|v: Value| is_variable(&v)));
    operations.insert(
        "quoted?",
        Operation::new(|v: Value| is_tagged_list(&v, "quote")),
    );
    operations.insert("application?", Operation::new(is_application));
    operations.insert(
        "assignment?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "set!")),
    );
    operations.insert(
        "definition?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "define")),
    );
    operations.insert(
        "if?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "if")),
    );
    operations.insert(
        "lambda?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "lambda")),
    );
    operations.insert(
        "begin?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "begin")),
    );
    operations.insert("text-of-quotation", Operation::new(CADR));
    operations.insert("lambda-parameters", Operation::new(CADR));
    operations.insert("lambda-body", Operation::new(CDDR));
    operations.insert(
        "make-procedure",
        Operation::new(|unev: Value, exp: Value, env: Value| {
            vec![Value::Symbol("procedure".into()), unev, exp, env]
        }),
    );
    operations.insert("operator", Operation::new(CAR));
    operations.insert("operands", Operation::new(CDR));
    operations.insert(
        "empty-arglist",
        Operation::new(|| Value::List(Vec::<Value>::new())),
    );
    operations.insert(
        "no-operands?",
        Operation::new(|pair: Value| is_null_pair(&pair)),
    );
    operations.insert("first-operand", Operation::new(CAR));
    operations.insert("last-operand?", Operation::new(is_last_one));
    operations.insert("rest-operands", Operation::new(CDR));
    operations.insert("adjoin-arg", Operation::new(adjoin_arg));
    operations.insert(
        "primitive-procedure?",
        Operation::new(|list: Value| Value::Symbol("primitive".into()) == list_ref(&list, 0)),
    );
    operations.insert(
        "compound-procedure?",
        Operation::new(|v: Value| is_compound_procedure(&v)),
    );
    operations.insert(
        "apply-primitive-procedure",
        Operation::new(apply_primitive_procedure),
    );
    operations.insert("procedure-parameters", Operation::new(CADR));
    operations.insert("procedure-body", Operation::new(CADDR));
    operations.insert("procedure-environment", Operation::new(CADDDR));
    operations.insert("begin-actions", Operation::new(CDR));
    operations.insert("first-exp", Operation::new(CAR));
    operations.insert("last-exp?", Operation::new(is_last_one));
    operations.insert("rest-exps", Operation::new(CDR));
    operations.insert("if-predicate", Operation::new(CADR));
    operations.insert("true?", Operation::new(is_boolean_true));
    operations.insert("if-alternative", Operation::new(if_alternative));
    operations.insert("if-consequent", Operation::new(CADDR));
    operations.insert("assignment-variable", Operation::new(CADR));
    operations.insert("assignment-value", Operation::new(CADDR));
    operations.insert("definition-variable", Operation::new(definition_variable));
    operations.insert("definition-value", Operation::new(definition_value));
    // support `cond` statement
    operations.insert(
        "cond?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "cond")),
    );
    operations.insert("cond-clauses", Operation::new(CDR));
    operations.insert("first-clause", Operation::new(CAR));
    operations.insert("last-clause?", Operation::new(is_last_one));
    operations.insert("rest-clauses", Operation::new(CDR));
    operations.insert("clause-action", Operation::new(CDR));
    operations.insert("clause-predicate", Operation::new(CAR));
    operations.insert(
        "else-clause?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "else")),
    );
    operations
}
