//! A parser of the register machine language.

use std::fmt;
use std::sync::Arc;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0, not_line_ending},
    combinator::{all_consuming, map, opt, recognize, verify},
    error::{ErrorKind, ParseError},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

/// RML Value
#[derive(Clone, Debug, PartialEq)]
pub enum RMLValue {
    Float(f64),
    Num(i32),
    List(Vec<RMLValue>),
    Str(String),
    Symbol(String),
}

impl fmt::Display for RMLValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float(v) => write!(f, "{}", v),
            Self::Num(v) => write!(f, "{}", v),
            Self::List(v) => write!(
                f,
                "({})",
                v.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Self::Str(v) => write!(f, "\"{}\"", v),
            Self::Symbol(v) => write!(f, "{}", v),
        }
    }
}

/// RML Syntax Tree
#[derive(Clone, Debug, PartialEq)]
pub enum RMLNode {
    Assignment(String, Arc<RMLNode>),
    Branch(Arc<RMLNode>),
    Constant(RMLValue),
    GotoLabel(Arc<RMLNode>),
    Label(String),
    List(Vec<RMLValue>),
    Operation(String, Vec<RMLNode>),
    PerformOp(Arc<RMLNode>),
    Reg(String),
    Restore(String),
    Save(String),
    Symbol(String),
    TestOp(Arc<RMLNode>),
}

impl fmt::Display for RMLNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assignment(reg, val) => write!(f, "(assign {} {})", reg, val),
            Self::Branch(label) => write!(f, "(branch {})", label),
            Self::Constant(value) => write!(f, "(const {})", value),
            Self::GotoLabel(label) => write!(f, "(goto {})", label),
            Self::Label(label) => write!(f, "(label {})", label),
            Self::List(v) => write!(
                f,
                "({})",
                v.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Self::Operation(op_name, args) => write!(
                f,
                "(op {}) {}",
                op_name,
                args.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Self::PerformOp(op) => write!(f, "(perform {})", op),
            Self::Reg(reg) => write!(f, "(reg {})", reg),
            Self::Restore(reg) => write!(f, "(restore {})", reg),
            Self::Save(reg) => write!(f, "(save {})", reg),
            Self::TestOp(op) => write!(f, "(test {})", op),
            Self::Symbol(v) => write!(f, "{}", v),
        }
    }
}

/// RML Parse Error
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum RMLParseError<I: fmt::Debug> {
    #[error("bad number")]
    BadNum,
    #[error("bad float point number")]
    BadFloatPoint,
    #[error("bad symbol")]
    BadSymbol,
    #[error("unknown parser error")]
    ParseFailure { input: I, kind: ErrorKind },
}

/// Take from [here](https://codeandbitters.com/lets-build-a-parser/#part-11-error-handling).
impl<I> ParseError<I> for RMLParseError<I>
where
    I: fmt::Debug,
{
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self::ParseFailure { input, kind }
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

type RMLResult<Rest, Expect> = IResult<Rest, Expect, RMLParseError<Rest>>;

pub fn parse(input: &str) -> Result<Vec<RMLNode>, RMLParseError<&str>> {
    let res = all_consuming(alt((rml_instructions, map(rml_instruction, |n| vec![n]))))(input);
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
        rml_const,
        rml_label,
        rml_reg,
        rml_branch,
        rml_goto,
        rml_save_and_restore,
        rml_apply_operation,
        rml_assign,
    )))(input)
    .or_else(|_| {
        map(sce(rml_symbol), |v| match v {
            RMLValue::Symbol(s) => RMLNode::Symbol(s),
            RMLValue::List(v) => RMLNode::List(v),
            _ => unreachable!(),
        })(input)
    })
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
fn rml_symbol(input: &str) -> RMLResult<&str, RMLValue> {
    map(valid_symbol, |s: &str| RMLValue::Symbol(s.into()))(input)
}

/// RML String
///
/// Any characters wrapped in double quotes, except the double-quote and backslash.
fn rml_string(input: &str) -> RMLResult<&str, RMLValue> {
    let parser = delimited(
        char('"'),
        take_while(|c| {
            let cv = c as u32;
            // 0x22: \", 0x5c: \\
            (cv != 0x22) && (cv != 0x5c)
        }),
        char('"'),
    );
    map(parser, |s: &str| RMLValue::Str(s.into()))(input)
}

/// RML Number
///
/// Valid syntax: -?\d+
fn rml_number(input: &str) -> RMLResult<&str, RMLValue> {
    let (remain, num_string) = recognize(pair(opt(tag("-")), digit1))(input)?;
    num_string.parse::<i32>().map_or_else(
        |_| Err(nom::Err::Failure(RMLParseError::BadNum)),
        |n| Ok((remain, RMLValue::Num(n))),
    )
}

/// RML Float Point Number
///
/// Valid syntax: -?\d+\.\d+
fn rml_float(input: &str) -> RMLResult<&str, RMLValue> {
    let (remain, float_num) = recognize(tuple((rml_number, char('.'), digit1)))(input)?;
    float_num.parse::<f64>().map_or_else(
        |_| Err(nom::Err::Failure(RMLParseError::BadFloatPoint)),
        |f| Ok((remain, RMLValue::Float(f))),
    )
}

/// RML List
///
/// Anything wrapped in double quotes.
fn rml_list(input: &str) -> RMLResult<&str, RMLValue> {
    let parser = delimited(sce(char('(')), many0(rml_value), sce(char(')')));
    map(parser, RMLValue::List)(input)
}

pub fn rml_value(input: &str) -> RMLResult<&str, RMLValue> {
    sce(alt((
        rml_float, rml_number, rml_symbol, rml_string, rml_list,
    )))(input)
}

/// RML Constant Value
///
/// Valid syntax:
/// - `(const "abc")` is the string `"abc"`,
/// - `(const abc)` is the symbol `abc`,
/// - `(const (a b c))` is the list `(a b c)`,
/// - `(const ())` is the empty list.
fn rml_const(input: &str) -> RMLResult<&str, RMLNode> {
    let parser = delimited(
        sce(char('(')),
        preceded(sce(tag("const")), rml_value),
        sce(char(')')),
    );
    map(parser, RMLNode::Constant)(input)
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
    map(parser, |l| RMLNode::Branch(Arc::new(l)))(input)
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
    map(parser, |l| RMLNode::GotoLabel(Arc::new(l)))(input)
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
    map(parser, |(name, args)| RMLNode::Operation(name.into(), args))(input)
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
        "test" => RMLNode::TestOp(Arc::new(op)),
        "perform" => RMLNode::PerformOp(Arc::new(op)),
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
        "restore" => RMLNode::Restore(reg.into()),
        "save" => RMLNode::Save(reg.into()),
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
        RMLNode::Assignment(reg.into(), Arc::new(value))
    })(input)
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_rml_symbol() {
        assert_eq!(
            Ok(("", RMLValue::Symbol("_1234".into()))),
            rml_symbol("_1234")
        );
        assert_eq!(
            Ok(("", RMLValue::Symbol("abcd".into()))),
            rml_symbol("abcd")
        );
        assert_eq!(
            Ok(("", RMLValue::Symbol("abcd?".into()))),
            rml_symbol("abcd?")
        );
        assert_eq!(
            Ok(("", RMLValue::Symbol("abcd!".into()))),
            rml_symbol("abcd!")
        );
        assert_eq!(
            Ok(("", RMLValue::Symbol("abcd-1234".into()))),
            rml_symbol("abcd-1234")
        );
        assert_eq!(
            Ok(("", RMLValue::Symbol("abcd_1234".into()))),
            rml_symbol("abcd_1234")
        );
        assert_eq!(
            Ok(("", RMLValue::Symbol("abcd_1234-".into()))),
            rml_symbol("abcd_1234-")
        );
        assert!(rml_symbol("1234").is_err());
        assert!(rml_symbol("-1234").is_err());
    }

    #[test]
    fn test_rml_string() {
        assert_eq!(Ok(("", RMLValue::Str("".into()))), rml_string(r#""""#));
        assert_eq!(
            Ok(("", RMLValue::Str("Hello".into()))),
            rml_string(r#""Hello""#)
        );
        assert_eq!(
            Ok(("", RMLValue::Str("Hello, world!".into()))),
            rml_string(r#""Hello, world!""#)
        );
        assert_eq!(
            Ok(("", RMLValue::Str("1+1=2".into()))),
            rml_string(r#""1+1=2""#)
        );
        assert_eq!(
            Ok(("", RMLValue::Str("1 + 1 = 2".into()))),
            rml_string(r#""1 + 1 = 2""#)
        );
        assert_eq!(Ok(("", RMLValue::Str(" ".into()))), rml_string(r#"" ""#));
    }

    #[test]
    fn test_rml_number() {
        assert_eq!(Ok(("", RMLValue::Num(42))), rml_number("42"));
        assert_eq!(Ok(("", RMLValue::Num(-42))), rml_number("-42"));
        assert_eq!(Ok(("_", RMLValue::Num(42))), rml_number("42_"));
        assert_eq!(Ok(("_2", RMLValue::Num(4))), rml_number("4_2"));
        assert!(rml_number("_42").is_err());
    }

    #[test]
    fn test_rml_float() {
        assert_eq!(Ok(("", RMLValue::Float(42.0))), rml_float("42.0"));
        assert_eq!(Ok(("", RMLValue::Float(-42.0))), rml_float("-42.0"));
        assert_eq!(Ok(("_", RMLValue::Float(42.0))), rml_float("42.0_"));
        assert!(rml_float("_42.0").is_err());
    }

    #[test]
    fn test_rml_list() {
        assert_eq!(
            Ok((
                "",
                RMLValue::List(vec![
                    RMLValue::Symbol("a".into()),
                    RMLValue::Symbol("b".into()),
                    RMLValue::Symbol("c".into())
                ])
            )),
            rml_list("(a b c)")
        );
        assert_eq!(
            Ok((
                "",
                RMLValue::List(vec![
                    RMLValue::Symbol("a".into()),
                    RMLValue::Symbol("b".into()),
                    RMLValue::Symbol("c".into())
                ])
            )),
            rml_list("( a  b    c     )")
        );
        assert_eq!(Ok(("", RMLValue::List(vec![]))), rml_list("()"));
        assert_eq!(
            Ok((
                "",
                RMLValue::List(vec![
                    RMLValue::Symbol("a".into()),
                    RMLValue::Num(0),
                    RMLValue::Float(1.0)
                ])
            )),
            rml_list("(a 0 1.0)")
        )
    }

    #[test]
    fn test_rml_const() {
        assert_eq!(
            Ok(("", RMLNode::Constant(RMLValue::Str("abc".into())))),
            rml_const(r#"(const "abc")"#)
        );
        assert_eq!(
            Ok(("", RMLNode::Constant(RMLValue::Symbol("abc".into())))),
            rml_const("(const abc)")
        );
        assert_eq!(
            Ok(("", RMLNode::Constant(RMLValue::Num(42)))),
            rml_const("(const 42)")
        );
        assert_eq!(
            Ok(("", RMLNode::Constant(RMLValue::Float(42.0)))),
            rml_const("(const 42.0)")
        );
        assert_eq!(
            Ok((
                "",
                RMLNode::Constant(RMLValue::List(vec![
                    RMLValue::Symbol("a".into()),
                    RMLValue::Symbol("b".into()),
                    RMLValue::Symbol("c".into())
                ]))
            )),
            rml_const("(const (a b c))")
        );
        assert_eq!(
            Ok(("", RMLNode::Constant(RMLValue::List(vec![])))),
            rml_const("(const ())")
        );
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
            Ok(("", RMLNode::Branch(Arc::new(RMLNode::Label("a".into()))))),
            rml_branch("(branch (label a))")
        );
    }

    #[test]
    fn test_rml_goto() {
        assert_eq!(
            Ok(("", RMLNode::GotoLabel(Arc::new(RMLNode::Label("a".into()))))),
            rml_goto("(goto (label a))")
        );
        assert_eq!(
            Ok(("", RMLNode::GotoLabel(Arc::new(RMLNode::Reg("a".into()))))),
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
        assert_eq!(
            Ok(("", RMLNode::Constant(RMLValue::Num(1)))),
            operation_arg("(const 1)")
        );
        assert_eq!(
            Ok(("", RMLNode::Constant(RMLValue::Symbol("abc".into())))),
            operation_arg("(const abc)")
        );
    }

    #[test]
    fn test_operation() {
        assert_eq!(
            Ok((
                "",
                RMLNode::Operation(
                    "add".into(),
                    vec![
                        RMLNode::Reg("a".into()),
                        RMLNode::Constant(RMLValue::Num(1))
                    ]
                )
            )),
            operation("(op add) (reg a) (const 1)")
        );
        assert_eq!(
            Ok(("", RMLNode::Operation("test".into(), vec![]))),
            operation("(op test)")
        );
    }

    #[test]
    fn test_rml_apply_operation() {
        assert_eq!(
            Ok((
                "",
                RMLNode::TestOp(Arc::new(RMLNode::Operation(
                    "add".into(),
                    vec![
                        RMLNode::Reg("a".into()),
                        RMLNode::Constant(RMLValue::Num(1))
                    ]
                )))
            )),
            rml_apply_operation("(test (op add) (reg a) (const 1))")
        );
        assert_eq!(
            Ok((
                "",
                RMLNode::TestOp(Arc::new(RMLNode::Operation(
                    "eq?".into(),
                    vec![
                        RMLNode::Reg("a".into()),
                        RMLNode::Constant(RMLValue::Num(1))
                    ]
                )))
            )),
            rml_apply_operation("(test (op eq?) (reg a) (const 1))")
        );
        assert_eq!(
            Ok((
                "",
                RMLNode::PerformOp(Arc::new(RMLNode::Operation("test".into(), vec![])))
            )),
            rml_apply_operation("(perform (op test))")
        );
    }

    #[test]
    fn test_rml_save_and_restore() {
        assert_eq!(
            Ok(("", RMLNode::Save("a".into()))),
            rml_save_and_restore("(save a)")
        );
        assert_eq!(
            Ok(("", RMLNode::Restore("a".into()))),
            rml_save_and_restore("(restore a)")
        );
    }

    #[test]
    fn test_rml_assign() {
        // (assign <register-name> (reg <register-name>))
        assert_eq!(
            Ok((
                "",
                RMLNode::Assignment("a".into(), Arc::new(RMLNode::Reg("b".into())))
            )),
            rml_assign("(assign a (reg b))"),
        );
        // (assign <register-name> (const <constant-value>))
        assert_eq!(
            Ok((
                "",
                RMLNode::Assignment("a".into(), Arc::new(RMLNode::Constant(RMLValue::Num(1))))
            )),
            rml_assign("(assign a (const 1))"),
        );
        // (assign <register-name> (op <operation-name>) <input_1> ... <input_n>)
        assert_eq!(
            Ok((
                "",
                RMLNode::Assignment(
                    "a".into(),
                    Arc::new(RMLNode::Operation(
                        "add".into(),
                        vec![
                            RMLNode::Reg("b".into()),
                            RMLNode::Constant(RMLValue::Num(1))
                        ]
                    ))
                )
            )),
            rml_assign("(assign a (op add) (reg b) (const 1))"),
        );
        // (assign <register-name> (label <label-name>))
        assert_eq!(
            Ok((
                "",
                RMLNode::Assignment("a".into(), Arc::new(RMLNode::Label("b".into())))
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
           (branch (label done))
           (assign m (const 42.0)))"#;
        let res = rml_instructions(instructions);
        assert_eq!(
            Ok((
                "",
                vec![
                    RMLNode::Symbol("controller".into()),
                    RMLNode::Assignment(
                        "n".into(),
                        Arc::new(RMLNode::Operation("read".into(), vec![]))
                    ),
                    RMLNode::TestOp(Arc::new(RMLNode::Operation(
                        "eq?".into(),
                        vec![
                            RMLNode::Reg("n".into()),
                            RMLNode::Constant(RMLValue::Symbol("q".into()))
                        ]
                    ))),
                    RMLNode::Branch(Arc::new(RMLNode::Label("done".into()))),
                    RMLNode::Assignment(
                        "m".into(),
                        Arc::new(RMLNode::Constant(RMLValue::Float(42.0)))
                    )
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
                RMLNode::PerformOp(Arc::new(RMLNode::Operation(
                    "print".into(),
                    vec![RMLNode::Constant(RMLValue::Str(
                        "Please enter a number or 'q' for quit: ".into()
                    ))]
                ))),
                RMLNode::Assignment(
                    "n".into(),
                    Arc::new(RMLNode::Operation("read".into(), vec![]))
                ),
                RMLNode::TestOp(Arc::new(RMLNode::Operation(
                    "eq?".into(),
                    vec![
                        RMLNode::Reg("n".into()),
                        RMLNode::Constant(RMLValue::Symbol("q".into()))
                    ]
                ))),
                RMLNode::Branch(Arc::new(RMLNode::Label("done".into()))),
                RMLNode::TestOp(Arc::new(RMLNode::Operation(
                    "noninteger?".into(),
                    vec![RMLNode::Reg("n".into())]
                ))),
                RMLNode::Branch(Arc::new(RMLNode::Label("controller".into()))),
                RMLNode::Assignment(
                    "continue".into(),
                    Arc::new(RMLNode::Label("fib-done".into()))
                ),
                RMLNode::Symbol("fib-loop".into()),
                RMLNode::TestOp(Arc::new(RMLNode::Operation(
                    "<".into(),
                    vec![
                        RMLNode::Reg("n".into()),
                        RMLNode::Constant(RMLValue::Num(2))
                    ]
                ))),
                RMLNode::Branch(Arc::new(RMLNode::Label("immediate-answer".into()))),
                RMLNode::Save("continue".into()),
                RMLNode::Assignment(
                    "continue".into(),
                    Arc::new(RMLNode::Label("afterfib-n-1".into()))
                ),
                RMLNode::Save("n".into()),
                RMLNode::Assignment(
                    "n".into(),
                    Arc::new(RMLNode::Operation(
                        "-".into(),
                        vec![
                            RMLNode::Reg("n".into()),
                            RMLNode::Constant(RMLValue::Num(1))
                        ]
                    ))
                ),
                RMLNode::GotoLabel(Arc::new(RMLNode::Label("fib-loop".into()))),
                RMLNode::Symbol("afterfib-n-1".into()),
                RMLNode::Restore("n".into()),
                RMLNode::Restore("continue".into()),
                RMLNode::Assignment(
                    "n".into(),
                    Arc::new(RMLNode::Operation(
                        "-".into(),
                        vec![
                            RMLNode::Reg("n".into()),
                            RMLNode::Constant(RMLValue::Num(2))
                        ]
                    ))
                ),
                RMLNode::Save("continue".into()),
                RMLNode::Assignment(
                    "continue".into(),
                    Arc::new(RMLNode::Label("afterfib-n-2".into()))
                ),
                RMLNode::Save("val".into()),
                RMLNode::GotoLabel(Arc::new(RMLNode::Label("fib-loop".into()))),
                RMLNode::Symbol("afterfib-n-2".into()),
                RMLNode::Assignment("n".into(), Arc::new(RMLNode::Reg("val".into()))),
                RMLNode::Restore("val".into()),
                RMLNode::Restore("continue".into()),
                RMLNode::Assignment(
                    "val".into(),
                    Arc::new(RMLNode::Operation(
                        "+".into(),
                        vec![RMLNode::Reg("val".into()), RMLNode::Reg("n".into())]
                    ))
                ),
                RMLNode::GotoLabel(Arc::new(RMLNode::Reg("continue".into()))),
                RMLNode::Symbol("immediate-answer".into()),
                RMLNode::Assignment("val".into(), Arc::new(RMLNode::Reg("n".into()))),
                RMLNode::GotoLabel(Arc::new(RMLNode::Reg("continue".into()))),
                RMLNode::Symbol("fib-done".into()),
                RMLNode::PerformOp(Arc::new(RMLNode::Operation(
                    "print-stack-statistics".into(),
                    vec![]
                ))),
                RMLNode::PerformOp(Arc::new(RMLNode::Operation(
                    "print".into(),
                    vec![RMLNode::Reg("val".into())]
                ))),
                RMLNode::PerformOp(Arc::new(RMLNode::Operation(
                    "initialize-stack".into(),
                    vec![]
                ))),
                RMLNode::GotoLabel(Arc::new(RMLNode::Label("controller".into()))),
                RMLNode::Symbol("done".into()),
            ]),
            res
        );
    }
}
