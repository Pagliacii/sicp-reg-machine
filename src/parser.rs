//! A parser of the register machine language.

use std::rc::Rc;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0, multispace1, not_line_ending},
    combinator::{all_consuming, map, opt, recognize, verify},
    error::{ErrorKind, ParseError},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

/// RML Syntax Tree
#[derive(Clone, Debug, PartialEq)]
pub enum RMLNode {
    Assignment((String, Rc<RMLNode>)),
    Branch(Rc<RMLNode>),
    GotoLabel(Rc<RMLNode>),
    List(Vec<RMLNode>),
    Label(String),
    Num(i64),
    Operation((String, Vec<RMLNode>)),
    PerformOp(Rc<RMLNode>),
    Reg(String),
    RestoreFrom(String),
    SaveTo(String),
    Str(String),
    Symbol(String),
    TestOp(Rc<RMLNode>),
}

/// RML Parse Error
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum RMLParseError {
    #[error("bad number")]
    BadNum,
    #[error("unknown parser error")]
    ParseFailure,
}

/// Take from [here](https://codeandbitters.com/lets-build-a-parser/#part-11-error-handling).
impl<I> ParseError<I> for RMLParseError {
    fn from_error_kind(_input: I, _kind: ErrorKind) -> Self {
        Self::ParseFailure
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

type RMLResult<Rest, Expect> = IResult<Rest, Expect, RMLParseError>;

pub fn parse(input: &str) -> Result<Vec<RMLNode>, RMLParseError> {
    let res = all_consuming(alt((map(rml_instruction, |n| vec![n]), rml_instructions)))(input);
    res.map(|(_, result)| Ok(result))
        .map_err(|nom_err| match nom_err {
            nom::Err::Error(e) | nom::Err::Failure(e) => e,
            _ => unreachable!(),
        })?
}

/// A combinator that takes a parser `inner` and produces a parser that also
/// consumes both leading and trailing whitespace, returning the output of `inner`.
/// Ref: [Nom Recipes](https://github.com/Geal/nom/blob/4028bb3276339b231a4c60f5486e117a3c81e479/doc/nom_recipes.md#L21-L46)
/// And ignore comments. `sce` stands for "spaces and comments eater".
fn sce<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(
        terminated(multispace0, opt(pair(tag(";"), not_line_ending))),
        terminated(inner, opt(pair(tag(";"), not_line_ending))),
        terminated(multispace0, opt(pair(tag(";"), not_line_ending))),
    )
}

/// Multiple RML instructions
fn rml_instructions(input: &str) -> RMLResult<&str, Vec<RMLNode>> {
    delimited(sce(char('(')), many0(rml_instruction), sce(char(')')))(input)
}

/// Single RML Instruction
fn rml_instruction(input: &str) -> RMLResult<&str, RMLNode> {
    sce(alt((
        rml_number,
        rml_string,
        rml_symbol,
        rml_const,
        rml_label,
        rml_reg,
        rml_branch,
        rml_goto,
        rml_save_and_restore,
        rml_apply_operation,
        rml_assign,
        rml_list,
    )))(input)
}

/// Valid symbol
///
/// Not all digits
fn valid_symbol(input: &str) -> RMLResult<&str, &str> {
    verify(
        take_while1(|c| {
            let cv = c as u32;
            // 0x00~0x1f: Control code, 0x20: space, 0x7f: DEL
            // 0x22: \", 0x27: ', 0x28: (, 0x29: )
            // 0x3b: ;, 0x40: `, 0x5c: \\, 0x7f: DEL
            match cv {
                0x00..=0x20 | 0x22 | 0x27..=0x29 | 0x3b | 0x40 | 0x5c => false,
                cv if cv >= 0x7e => false,
                _ => true,
            }
        }),
        |s: &str| s.parse::<f64>().is_err(),
    )(input)
}

/// RML Symbol
///
/// For the controller label.
fn rml_symbol(input: &str) -> RMLResult<&str, RMLNode> {
    map(valid_symbol, |s: &str| RMLNode::Symbol(s.into()))(input)
}

/// RML String
///
/// Any characters wrapped in double quotes, except the double-quote and backslash.
fn rml_string(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        char('"'),
        take_while(|c| {
            let cv = c as u32;
            // 0x22: \", 0x5c: \\
            (cv != 0x22) && (cv != 0x5c)
        }),
        char('"'),
    );
    map(parser, |s: &str| RMLNode::Str(s.to_string()))(input)
}

fn const_value(input: &str) -> RMLResult<&str, RMLNode> {
    sce(alt((rml_number, rml_symbol, rml_string, rml_list)))(input)
}

/// RML Constant Value
///
/// Valid syntax:
/// - `(const "abc")` is the string `"abc"`,
/// - `(const abc)` is the symbol `abc`,
/// - `(const (a b c))` is the list `(a b c)`,
/// - `(const ())` is the empty list.
fn rml_const(input: &str) -> RMLResult<&str, RMLNode> {
    delimited(
        sce(char('(')),
        preceded(sce(tag("const")), const_value),
        sce(char(')')),
    )(input)
}

/// RML Number
///
/// Valid syntax: -?\d+
fn rml_number(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = recognize(pair(opt(tag("-")), digit1));
    // TODO: Parse error handling
    map(parser, |s: &str| RMLNode::Num(s.parse::<i64>().unwrap()))(input)
}

/// RML List
///
/// Anything wrapped in double quotes.
fn rml_list(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        sce(char('(')),
        separated_list0(multispace1, rml_symbol),
        sce(char(')')),
    );
    map(parser, |v| RMLNode::List(v))(input)
}

/// RML Reg Instruction
///
/// Get the register contents by name.
/// Valid syntax: `(reg <register-name>)`
fn rml_reg(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        sce(char('(')),
        preceded(sce(tag("reg")), valid_symbol),
        sce(char(')')),
    );
    map(parser, |s| RMLNode::Reg(s.into()))(input)
}

/// RML Label Instruction
///
/// Label name to jump to.
/// Valid syntax: `(label <label-name>)`
fn rml_label(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        sce(char('(')),
        preceded(sce(tag("label")), valid_symbol),
        sce(char(')')),
    );
    map(parser, |n| RMLNode::Label(n.into()))(input)
}

/// RML Branch Instruction
///
/// A conditional branch to a location indicated by a controller label,
/// based on the result of the previous test. If the test is false,
/// the controller should continue with the next instruction in the sequence.
/// Otherwise, the controller should continue with the instruction after the label.
/// Valid syntax: `(branch (label <label-name>))`
fn rml_branch(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        sce(char('(')),
        preceded(sce(tag("branch")), rml_label),
        sce(char(')')),
    );
    map(parser, |l| RMLNode::Branch(Rc::new(l)))(input)
}

/// RML Goto Instruction
///
/// An unconditional branch naming a controller label at which to continue execution.
/// Valid syntax:
/// - `(goto (label <label-name>))`
/// - `(goto (reg <register-name>))`
fn rml_goto(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        sce(char('(')),
        preceded(sce(tag("goto")), alt((rml_label, rml_reg))),
        sce(char(')')),
    );
    map(parser, |l| RMLNode::GotoLabel(Rc::new(l)))(input)
}

/// Operation name
///
/// Valid syntax: `(op <operation-name>)`
fn operation_name(input: &str) -> RMLResult<&str, &str> {
    delimited(
        sce(char('(')),
        preceded(sce(tag("op")), valid_symbol),
        sce(char(')')),
    )(input)
}

/// Operation arguments
///
/// Valid syntax: `(reg <register-name>)` or `(const <constant-value>)`
fn operation_arg(input: &str) -> RMLResult<&str, RMLNode> {
    sce(alt((rml_const, rml_reg)))(input)
}

/// RML Operation
///
/// Combines operation name and arguments
/// Valid syntax: `(op <operation-name>) <input_1> ... <input_n>`
fn operation(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = pair(operation_name, many0(operation_arg));
    map(parser, |(name, args)| {
        RMLNode::Operation((name.into(), args))
    })(input)
}

/// RML Instructions applying operations
///
/// Valid syntax:
/// - `(perform (op <operation-name>) <input_1> ... <input_n>)`
/// - `(test (op <operation-name>) <input_1> ... <input_n>)`
fn rml_apply_operation(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        sce(char('(')),
        pair(sce(alt((tag("test"), tag("perform")))), operation),
        sce(char(')')),
    );
    map(parser, |(inst, op)| match inst {
        "test" => RMLNode::TestOp(Rc::new(op)),
        "perform" => RMLNode::PerformOp(Rc::new(op)),
        _ => unreachable!(),
    })(input)
}

/// RML Instructions manipulating the stack
///
/// Valid syntax:
/// - `(save <register-name>)`: save the contents of specified register on the stack.
/// - `(restore <register-name>)`: pop the top item of stack, and save to the specified register.
fn rml_save_and_restore(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        sce(char('(')),
        pair(sce(alt((tag("save"), tag("restore")))), valid_symbol),
        sce(char(')')),
    );
    map(parser, |(inst, reg)| match inst {
        "restore" => RMLNode::RestoreFrom(reg.into()),
        "save" => RMLNode::SaveTo(reg.into()),
        _ => unreachable!(),
    })(input)
}

/// RML Assign Instruction
///
/// Assigns values to the register.
/// Valid syntax:
/// - `(assign <register-name> (reg <register-name>))`
/// - `(assign <register-name> (const <constant-value>))`
/// - `(assign <register-name> (op <operation-name>) <input_1> ... <input_n>)`
/// - `(assign <register-name> (label <label-name>))`
fn rml_assign(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        sce(char('(')),
        preceded(
            sce(tag("assign")),
            pair(
                sce(valid_symbol),
                alt((rml_const, rml_reg, rml_label, operation)),
            ),
        ),
        sce(char(')')),
    );
    map(parser, |(reg, value)| {
        RMLNode::Assignment((reg.into(), Rc::new(value)))
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rml_symbol() {
        assert_eq!(
            Ok(("", RMLNode::Symbol("_1234".into()))),
            rml_symbol("_1234")
        );
        assert_eq!(Ok(("", RMLNode::Symbol("abcd".into()))), rml_symbol("abcd"));
        assert_eq!(
            Ok(("", RMLNode::Symbol("abcd?".into()))),
            rml_symbol("abcd?")
        );
        assert_eq!(
            Ok(("", RMLNode::Symbol("abcd!".into()))),
            rml_symbol("abcd!")
        );
        assert_eq!(
            Ok(("", RMLNode::Symbol("abcd-1234".into()))),
            rml_symbol("abcd-1234")
        );
        assert_eq!(
            Ok(("", RMLNode::Symbol("abcd_1234".into()))),
            rml_symbol("abcd_1234")
        );
        assert_eq!(
            Ok(("", RMLNode::Symbol("abcd_1234-".into()))),
            rml_symbol("abcd_1234-")
        );
        assert!(rml_symbol("1234").is_err());
        assert!(rml_symbol("-1234").is_err());
    }

    #[test]
    fn test_rml_string() {
        assert_eq!(Ok(("", RMLNode::Str("".into()))), rml_string(r#""""#));
        assert_eq!(
            Ok(("", RMLNode::Str("Hello".into()))),
            rml_string(r#""Hello""#)
        );
        assert_eq!(
            Ok(("", RMLNode::Str("Hello, world!".into()))),
            rml_string(r#""Hello, world!""#)
        );
        assert_eq!(
            Ok(("", RMLNode::Str("1+1=2".into()))),
            rml_string(r#""1+1=2""#)
        );
        assert_eq!(
            Ok(("", RMLNode::Str("1 + 1 = 2".into()))),
            rml_string(r#""1 + 1 = 2""#)
        );
        assert_eq!(Ok(("", RMLNode::Str(" ".into()))), rml_string(r#"" ""#));
    }

    #[test]
    fn test_rml_number() {
        assert_eq!(Ok(("", RMLNode::Num(42))), rml_number("42"));
        assert_eq!(Ok(("", RMLNode::Num(-42))), rml_number("-42"));
        assert_eq!(Ok(("_", RMLNode::Num(42))), rml_number("42_"));
        assert_eq!(Ok(("_2", RMLNode::Num(4))), rml_number("4_2"));
        assert!(rml_number("_42").is_err());
    }

    #[test]
    fn test_rml_list() {
        assert_eq!(
            Ok((
                "",
                RMLNode::List(vec![
                    RMLNode::Symbol("a".into()),
                    RMLNode::Symbol("b".into()),
                    RMLNode::Symbol("c".into())
                ])
            )),
            rml_list("(a b c)")
        );
        assert_eq!(
            Ok((
                "",
                RMLNode::List(vec![
                    RMLNode::Symbol("a".into()),
                    RMLNode::Symbol("b".into()),
                    RMLNode::Symbol("c".into())
                ])
            )),
            rml_list("( a  b    c     )")
        );
        assert_eq!(Ok(("", RMLNode::List(vec![]))), rml_list("()"));
    }

    #[test]
    fn test_rml_const() {
        assert_eq!(
            Ok(("", RMLNode::Str("abc".into()))),
            rml_const(r#"(const "abc")"#)
        );
        assert_eq!(
            Ok(("", RMLNode::Symbol("abc".into()))),
            rml_const("(const abc)")
        );
        assert_eq!(Ok(("", RMLNode::Num(42))), rml_const("(const 42)"));
        assert_eq!(
            Ok((
                "",
                RMLNode::List(vec![
                    RMLNode::Symbol("a".into()),
                    RMLNode::Symbol("b".into()),
                    RMLNode::Symbol("c".into())
                ])
            )),
            rml_const("(const (a b c))")
        );
        assert_eq!(Ok(("", RMLNode::List(vec![]))), rml_const("(const ())"));
    }

    #[test]
    fn test_rml_reg() {
        assert_eq!(Ok(("", RMLNode::Reg("a".into()))), rml_reg("(reg a)"));
        assert_eq!(Ok(("", RMLNode::Reg("a1".into()))), rml_reg("(reg a1)"));
        assert_eq!(
            Ok(("", RMLNode::Reg("_1234".into()))),
            rml_reg("(reg _1234)")
        );
        assert!(rml_reg("(reg 123)").is_err());
    }

    #[test]
    fn test_rml_label() {
        assert_eq!(
            Ok(("", RMLNode::Label("branch1".into()))),
            rml_label("(label branch1)")
        );
        assert_eq!(
            Ok(("", RMLNode::Label("branch-2".into()))),
            rml_label("(label branch-2)")
        );
        assert_eq!(
            Ok(("", RMLNode::Label("branch_3".into()))),
            rml_label("(label branch_3)")
        );
    }

    #[test]
    fn test_rml_branch() {
        assert_eq!(
            Ok(("", RMLNode::Branch(Rc::new(RMLNode::Label("a".into()))))),
            rml_branch("(branch (label a))")
        );
    }

    #[test]
    fn test_rml_goto() {
        assert_eq!(
            Ok(("", RMLNode::GotoLabel(Rc::new(RMLNode::Label("a".into()))))),
            rml_goto("(goto (label a))")
        );
        assert_eq!(
            Ok(("", RMLNode::GotoLabel(Rc::new(RMLNode::Reg("a".into()))))),
            rml_goto("(goto (reg a))")
        );
    }

    #[test]
    fn test_operation_name() {
        assert_eq!(Ok(("", "add")), operation_name("(op add)"));
        assert_eq!(Ok(("", "a1")), operation_name("(op a1)"));
        assert_eq!(Ok(("", "_1234")), operation_name("(op _1234)"));
        assert!(operation_name("(op 123)").is_err());
    }

    #[test]
    fn test_operation_arg() {
        assert_eq!(Ok(("", RMLNode::Reg("a".into()))), operation_arg("(reg a)"));
        assert_eq!(Ok(("", RMLNode::Num(1))), operation_arg("(const 1)"));
        assert_eq!(
            Ok(("", RMLNode::Symbol("abc".into()))),
            operation_arg("(const abc)")
        );
    }

    #[test]
    fn test_operation() {
        assert_eq!(
            Ok((
                "",
                RMLNode::Operation((
                    "add".into(),
                    vec![RMLNode::Reg("a".into()), RMLNode::Num(1)]
                )),
            )),
            operation("(op add) (reg a) (const 1)")
        );
        assert_eq!(
            Ok(("", RMLNode::Operation(("test".into(), vec![])),)),
            operation("(op test)")
        );
    }

    #[test]
    fn test_rml_apply_operation() {
        assert_eq!(
            Ok((
                "",
                RMLNode::TestOp(Rc::new(RMLNode::Operation((
                    "add".into(),
                    vec![RMLNode::Reg("a".into()), RMLNode::Num(1)]
                )))),
            )),
            rml_apply_operation("(test (op add) (reg a) (const 1))")
        );
        assert_eq!(
            Ok((
                "",
                RMLNode::TestOp(Rc::new(RMLNode::Operation((
                    "eq?".into(),
                    vec![RMLNode::Reg("a".into()), RMLNode::Num(1)]
                )))),
            )),
            rml_apply_operation("(test (op eq?) (reg a) (const 1))")
        );
        assert_eq!(
            Ok((
                "",
                RMLNode::PerformOp(Rc::new(RMLNode::Operation(("test".into(), vec![])),))
            )),
            rml_apply_operation("(perform (op test))")
        );
    }

    #[test]
    fn test_rml_save_and_restore() {
        assert_eq!(
            Ok(("", RMLNode::SaveTo("a".into()))),
            rml_save_and_restore("(save a)")
        );
        assert_eq!(
            Ok(("", RMLNode::RestoreFrom("a".into()))),
            rml_save_and_restore("(restore a)")
        );
    }

    #[test]
    fn test_rml_assign() {
        // (assign <register-name> (reg <register-name>))
        assert_eq!(
            Ok((
                "",
                RMLNode::Assignment(("a".into(), Rc::new(RMLNode::Reg("b".into()))))
            )),
            rml_assign("(assign a (reg b))"),
        );
        // (assign <register-name> (const <constant-value>))
        assert_eq!(
            Ok((
                "",
                RMLNode::Assignment(("a".into(), Rc::new(RMLNode::Num(1))))
            )),
            rml_assign("(assign a (const 1))"),
        );
        // (assign <register-name> (op <operation-name>) <input_1> ... <input_n>)
        assert_eq!(
            Ok((
                "",
                RMLNode::Assignment((
                    "a".into(),
                    Rc::new(RMLNode::Operation((
                        "add".into(),
                        vec![RMLNode::Reg("b".into()), RMLNode::Num(1)]
                    )))
                ))
            )),
            rml_assign("(assign a (op add) (reg b) (const 1))"),
        );
        // (assign <register-name> (label <label-name>))
        assert_eq!(
            Ok((
                "",
                RMLNode::Assignment(("a".into(), Rc::new(RMLNode::Label("b".into()))))
            )),
            rml_assign("(assign a (label b))"),
        );
    }

    #[test]
    fn test_rml_instructions() {
        let instructions = r#"
        (controller
           ;;; comments
           (assign n (op read))  ; inline comment
           (test (op eq?) (reg n) (const q))
           (branch (label done)))"#;
        let res = rml_instructions(instructions);
        assert_eq!(
            Ok((
                "",
                vec![
                    RMLNode::Symbol("controller".into()),
                    RMLNode::Assignment((
                        "n".into(),
                        Rc::new(RMLNode::Operation(("read".into(), vec![])))
                    )),
                    RMLNode::TestOp(Rc::new(RMLNode::Operation((
                        "eq?".into(),
                        vec![RMLNode::Reg("n".into()), RMLNode::Symbol("q".into())]
                    )))),
                    RMLNode::Branch(Rc::new(RMLNode::Label("done".into()))),
                ]
            )),
            res
        );
    }

    #[test]
    fn test_parse() {
        let instructions = std::str::from_utf8(include_bytes!("../tests/rml_insts.scm")).unwrap();
        let res = parse(instructions);
        assert!(res.is_ok());
        assert_eq!(
            Ok(vec![
                RMLNode::Symbol("controller".into()),
                RMLNode::PerformOp(Rc::new(RMLNode::Operation((
                    "print".into(),
                    vec![RMLNode::Str(
                        "Please enter a number or 'q' for quit: ".into()
                    )]
                )))),
                RMLNode::Assignment((
                    "n".into(),
                    Rc::new(RMLNode::Operation(("read".into(), vec![])))
                )),
                RMLNode::TestOp(Rc::new(RMLNode::Operation((
                    "eq?".into(),
                    vec![RMLNode::Reg("n".into()), RMLNode::Symbol("q".into())]
                )))),
                RMLNode::Branch(Rc::new(RMLNode::Label("done".into()))),
                RMLNode::TestOp(Rc::new(RMLNode::Operation((
                    "noninteger?".into(),
                    vec![RMLNode::Reg("n".into())]
                )))),
                RMLNode::Branch(Rc::new(RMLNode::Label("controller".into()))),
                RMLNode::Assignment((
                    "continue".into(),
                    Rc::new(RMLNode::Label("fib-done".into()))
                )),
                RMLNode::Symbol("fib-loop".into()),
                RMLNode::TestOp(Rc::new(RMLNode::Operation((
                    "<".into(),
                    vec![RMLNode::Reg("n".into()), RMLNode::Num(2)]
                )))),
                RMLNode::Branch(Rc::new(RMLNode::Label("immediate-answer".into()))),
                RMLNode::SaveTo("continue".into()),
                RMLNode::Assignment((
                    "continue".into(),
                    Rc::new(RMLNode::Label("afterfib-n-1".into()))
                )),
                RMLNode::SaveTo("n".into()),
                RMLNode::Assignment((
                    "n".into(),
                    Rc::new(RMLNode::Operation((
                        "-".into(),
                        vec![RMLNode::Reg("n".into()), RMLNode::Num(1)]
                    )))
                )),
                RMLNode::GotoLabel(Rc::new(RMLNode::Label("fib-loop".into()))),
                RMLNode::Symbol("afterfib-n-1".into()),
                RMLNode::RestoreFrom("n".into()),
                RMLNode::RestoreFrom("continue".into()),
                RMLNode::Assignment((
                    "n".into(),
                    Rc::new(RMLNode::Operation((
                        "-".into(),
                        vec![RMLNode::Reg("n".into()), RMLNode::Num(2)]
                    )))
                )),
                RMLNode::SaveTo("continue".into()),
                RMLNode::Assignment((
                    "continue".into(),
                    Rc::new(RMLNode::Label("afterfib-n-2".into()))
                )),
                RMLNode::SaveTo("val".into()),
                RMLNode::GotoLabel(Rc::new(RMLNode::Label("fib-loop".into()))),
                RMLNode::Symbol("afterfib-n-2".into()),
                RMLNode::Assignment(("n".into(), Rc::new(RMLNode::Reg("val".into())))),
                RMLNode::RestoreFrom("val".into()),
                RMLNode::RestoreFrom("continue".into()),
                RMLNode::Assignment((
                    "val".into(),
                    Rc::new(RMLNode::Operation((
                        "+".into(),
                        vec![RMLNode::Reg("val".into()), RMLNode::Reg("n".into())]
                    )))
                )),
                RMLNode::GotoLabel(Rc::new(RMLNode::Reg("continue".into()))),
                RMLNode::Symbol("immediate-answer".into()),
                RMLNode::Assignment(("val".into(), Rc::new(RMLNode::Reg("n".into())))),
                RMLNode::GotoLabel(Rc::new(RMLNode::Reg("continue".into()))),
                RMLNode::Symbol("fib-done".into()),
                RMLNode::PerformOp(Rc::new(RMLNode::Operation((
                    "print-stack-statistics".into(),
                    vec![]
                )))),
                RMLNode::PerformOp(Rc::new(RMLNode::Operation((
                    "print".into(),
                    vec![RMLNode::Reg("val".into())]
                )))),
                RMLNode::PerformOp(Rc::new(RMLNode::Operation((
                    "initialize-stack".into(),
                    vec![]
                )))),
                RMLNode::GotoLabel(Rc::new(RMLNode::Label("controller".into()))),
                RMLNode::Symbol("done".into()),
            ]),
            res
        );
    }
}
