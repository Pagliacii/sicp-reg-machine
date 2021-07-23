use reg_machine::machine::{
    procedure::Procedure,
    value::{ToValue, TryFromValue, Value},
};

use super::supports::{
    environment::{get_global_environment, manipulate_env},
    io::{announce_output, prompt_for_input, read, user_print},
    list::*,
    primitive::apply_primitive_procedure,
    syntax::*,
};

// For convenience
fn tag_checker(name: &'static str, tag: &'static str) -> Procedure {
    Procedure::new(name, 1, move |args| is_tagged_list(&args[0], tag))
}

pub fn operations() -> Vec<Procedure> {
    // Same behavior likes the same name procedure in Scheme.
    let car = Procedure::new("car", 1, |args| list_ref(&args[0], 0));
    let cadr = Procedure::new("cadr", 1, |args| list_ref(&args[0], 1));
    let caddr = Procedure::new("caddr", 1, |args| list_ref(&args[0], 2));
    let cadddr = Procedure::new("cadddr", 1, |args| list_ref(&args[0], 3));
    let cdr = Procedure::new("cdr", 1, |args| list_rest(&args[0], 1));
    let cddr = Procedure::new("cdr", 1, |args| list_rest(&args[0], 2));
    // For convenience
    let is_last_one = Procedure::new("is_last_one", 1, |args| is_last_one(&args[0]));

    let mut operations: Vec<Procedure> = vec![];
    operations.push(Procedure::new("read", 0, |_| read()));
    operations.push(Procedure::new("debug", 1, |args| println!("{:?}", args[0])));
    operations.push(Procedure::new("prompt-for-input", 1, |args| {
        prompt_for_input(&args[0])
    }));
    operations.push(Procedure::new("announce-output", 1, |args| {
        announce_output(&args[0])
    }));
    operations.push(Procedure::new("user-print", 1, |args| user_print(&args[0])));
    operations.push(Procedure::new("get-global-environment", 0, |_| {
        get_global_environment()
    }));
    operations.push(Procedure::new("lookup-variable-value", 2, |args| {
        manipulate_env("lookup", &args[1], &args[..1])
    }));
    operations.push(Procedure::new("set-variable-value!", 3, |args| {
        manipulate_env("update", &args[2], &args[..2])
    }));
    operations.push(Procedure::new("extend-environment", 3, |args| {
        manipulate_env("extend", &args[2], &args[..2])
    }));
    operations.push(Procedure::new("define-variable!", 3, |args| {
        manipulate_env("define", &args[2], &args[..2]);
    }));
    operations.push(Procedure::new("self-evaluating?", 1, |args| {
        args[0].is_num() || args[0].is_string()
    }));
    operations.push(Procedure::new("variable?", 1, |args| args[0].is_symbol()));
    operations.push(Procedure::new("application?", 1, |args| {
        !args[0].is_empty_list()
    }));
    operations.push(tag_checker("quoted?", "quote"));
    operations.push(tag_checker("assignment?", "set!"));
    operations.push(tag_checker("definition?", "define"));
    operations.push(tag_checker("if?", "if"));
    operations.push(tag_checker("lambda?", "lambda"));
    operations.push(tag_checker("begin?", "begin"));
    operations.push(Procedure::new("make-procedure", 3, |args| {
        let mut proc = args[..3].to_vec();
        proc.insert(0, Value::new("procedure"));
        proc
    }));
    operations.push(Procedure::new("empty-arglist", 0, |_| Value::empty_list()));
    operations.push(Procedure::duplicate(&cadr, "text-of-quotation"));
    operations.push(Procedure::duplicate(&cadr, "lambda-parameters"));
    operations.push(Procedure::duplicate(&cddr, "lambda-body"));
    operations.push(Procedure::duplicate(&car, "operator"));
    operations.push(Procedure::duplicate(&cdr, "operands"));
    operations.push(Procedure::new("no-operands?", 1, |args| {
        args[0].is_empty_list()
    }));
    operations.push(Procedure::duplicate(&car, "first-operand"));
    operations.push(Procedure::duplicate(&is_last_one, "last-operand?"));
    operations.push(Procedure::duplicate(&cdr, "rest-operands"));
    operations.push(Procedure::new("adjoin-arg", 2, |args| {
        adjoin_arg(&args[0], &args[1])
    }));
    operations.push(Procedure::new("primitive-procedure?", 1, |args| {
        Value::Symbol("primitive".into()) == list_ref(&args[0], 0)
    }));
    operations.push(Procedure::new("compound-procedure?", 1, |args| {
        is_compound_procedure(&args[0])
    }));
    operations.push(Procedure::new("apply-primitive-procedure", 2, |args| {
        let proc = Vec::<Value>::try_from(&args[0]).unwrap();
        let args = Vec::<Value>::try_from(&args[1]).unwrap();
        apply_primitive_procedure(proc, args)
    }));
    operations.push(Procedure::duplicate(&cadr, "procedure-parameters"));
    operations.push(Procedure::duplicate(&caddr, "procedure-body"));
    operations.push(Procedure::duplicate(&cadddr, "procedure-environment"));
    operations.push(Procedure::duplicate(&cdr, "begin-actions"));
    operations.push(Procedure::duplicate(&car, "first-exp"));
    operations.push(Procedure::duplicate(&is_last_one, "last-exp?"));
    operations.push(Procedure::duplicate(&cdr, "rest-exps"));
    operations.push(Procedure::duplicate(&cadr, "if-predicate"));
    operations.push(Procedure::new("true?", 1, |args| args[0].is_true()));
    operations.push(Procedure::new("if-alternative", 1, |args| {
        if_alternative(&args[0])
    }));
    operations.push(Procedure::duplicate(&caddr, "if-consequent"));
    operations.push(Procedure::duplicate(&cadr, "assignment-variable"));
    operations.push(Procedure::duplicate(&caddr, "assignment-value"));
    operations.push(Procedure::new("definition-variable", 1, |args| {
        definition_variable(&args[0])
    }));
    operations.push(Procedure::new("definition-value", 1, |args| {
        definition_value(&args[0])
    }));
    // support `cond` statement
    operations.push(tag_checker("cond?", "cond"));
    operations.push(Procedure::duplicate(&cdr, "cond-clauses"));
    operations.push(Procedure::duplicate(&car, "first-clause"));
    operations.push(Procedure::duplicate(&is_last_one, "last-clause?"));
    operations.push(Procedure::duplicate(&cdr, "rest-clauses"));
    operations.push(Procedure::duplicate(&cdr, "clause-action"));
    operations.push(Procedure::duplicate(&car, "clause-predicate"));
    operations.push(tag_checker("else-clause?", "else"));
    // support `let` statement, as a syntactic sugar
    operations.push(tag_checker("let?", "let"));
    operations.push(Procedure::new("let->combination", 1, |args| {
        // `(let ((<var_1> <exp_1>) ... (<var_n> <exp_n>)) <body>)`
        let exp = Vec::<Value>::try_from(&args[0]).unwrap();
        // `((<var_1> <exp_1>) ... (<var_n> <exp_n>))`
        let var_pairs = Vec::<Value>::try_from(&exp[1]).unwrap();
        let mut vars: Vec<Value> = vec![];
        let mut exps: Vec<Value> = vec![];
        for pair in var_pairs.iter() {
            // pair: (<var_n> <exp_n>)
            let pair = Vec::<Value>::try_from(pair).unwrap();
            vars.push(pair[0].clone());
            exps.push(pair[1].clone());
        }
        // => `(lambda (<var_1> ... <var_n>) <body>)`
        let lambda = vec!["lambda".to_value(), vars.to_value(), exp[2].clone()];
        // => `((lambda (<var_1> ... <var_n>) <body>) <exp_1> ... <exp_2>)`
        exps.insert(0, lambda.to_value());
        exps
    }));
    operations
}
