use std::io::{self, prelude::*};

use fancy_regex::Regex;
use lazy_static::lazy_static;
use log::debug;
use reg_machine::{machine::value::Value, parser::rml_value, rmlvalue_to_value};

use super::{list::list_ref, syntax::is_compound_procedure};

/// Read from Stdin and replace `'` to `quote`.
/// Supports multiple lines.
pub fn read() -> Value {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"'(\([^'\)]*\)|\w+)+(?!')").unwrap();
    }
    let mut balance = 0;
    let mut result = String::new();
    let mut previous = 0u8 as char;

    // Read multiple lines and balance parentheses.
    for b in io::stdin().bytes() {
        let mut c = b.unwrap() as char;
        if c == '(' {
            balance += 1;
        } else if c == ')' {
            balance -= 1;
        } else if c == '\n' {
            if balance == 0 {
                break;
            } else {
                c = ' ';
            }
        } else if c == ' ' && c == previous {
            continue;
        }
        previous = c;
        result.push(c);
    }

    debug!("read result: {}", result);
    let (_, res) = rml_value(&RE.replace_all(&result, "(quote $1)")).unwrap();
    rmlvalue_to_value(&res)
}

pub fn display(val: Value) {
    let s = match val {
        Value::String(v) => v,
        others => others.to_string(),
    };
    print!("{}", s);
}

pub fn prompt_for_input(val: Value) {
    println!();
    display(val);
    println!();
}

pub fn announce_output(val: Value) {
    println!();
    display(val);
    println!();
}

pub fn user_print(s: Value) {
    if is_compound_procedure(&s) {
        println!(
            "(compound-procedure {} {} <procedure-env>)",
            list_ref(&s, 1),
            list_ref(&s, 2),
        );
    } else {
        println!("{}", s);
    }
}
