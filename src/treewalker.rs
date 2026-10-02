use std::{collections::HashMap, fmt::Display, io::Write};

use ordered_float::NotNan;

use crate::{
    checker::undeclared_variable_info,
    parser::{ASTNode, ASTNodeType, Atom, BinOpType, Expr, ExprType, SpanType, UnaryOpType},
    types::{Info, InterpreterIO, Span},
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Value {
    String(String),
    Int(i32),
    Bool(bool),
    Real(NotNan<f64>),
    Nothing,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum Type {
    String,
    Int,
    Bool,
    Char,
    Real,
}
pub fn print_ty_option(t: Option<Type>) -> String {
    t.map_or(String::from("nothing"), |i| i.to_string())
}
impl Value {
    fn ty(&self) -> Type {
        match self {
            Value::String(_) => Type::String,
            Value::Int(_) => Type::Int,
            Value::Bool(_) => Type::Bool,
            Value::Real(_) => Type::Real,
            Value::Nothing => unreachable!("attempted getting the type of Nothing"),
        }
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::String => write!(f, "STRING"),
            Type::Int => write!(f, "INTEGER"),
            Type::Bool => write!(f, "BOOLEAN"),
            Type::Char => write!(f, "CHAR"),
            Type::Real => write!(f, "REAL"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Variable {
    name: String,
    declared_ty: Type,
    value: Option<Value>,
}
pub struct Constant {
    value: (Atom, Span),
}
pub struct Treewalker<I: InterpreterIO> {
    nodes: Vec<ASTNode>,
    variables: HashMap<String, Variable>,
    constants: HashMap<String, Constant>,
    io: I,
}
#[derive(Debug, Clone, Copy)]
pub enum RuntimeErrorType {
    TypeError,
    ReferenceError,
    DivideByZeroError,
    IOError,
    InternalError,
}
impl Display for RuntimeErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeErrorType::TypeError => write!(f, "type error"),
            RuntimeErrorType::ReferenceError => write!(f, "reference error"),
            RuntimeErrorType::DivideByZeroError => write!(f, "divide by zero error"),
            RuntimeErrorType::IOError => write!(f, "io error"),
            RuntimeErrorType::InternalError => write!(f, "!! INTERNAL ERROR !!"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub msg: String,
    pub span: SpanType,
    pub ty: RuntimeErrorType,
    pub info: Vec<Info>,
}

trait AsValue {
    fn as_value(&self) -> Value;
}

impl AsValue for i32 {
    fn as_value(&self) -> Value {
        Value::Int(*self)
    }
}

impl AsValue for String {
    fn as_value(&self) -> Value {
        Value::String(self.to_owned())
    }
}

impl AsValue for bool {
    fn as_value(&self) -> Value {
        Value::Bool(self.to_owned())
    }
}

impl AsValue for NotNan<f64> {
    fn as_value(&self) -> Value {
        Value::Real(self.to_owned())
    }
}

// TODO: rest of the AsValue impls

fn var_not_decl_err(name: &str, span: SpanType) -> Box<RuntimeError> {
    Box::new(RuntimeError {
        msg: format!("cannot find variable {name}"),
        span,
        ty: RuntimeErrorType::ReferenceError,
        info: undeclared_variable_info(name.to_owned(), None),
    })
}
// note that these conversion funcs aren't complete, except for to_str()
fn to_str(val: Value) -> String {
    match val {
        Value::String(s) => s,
        Value::Int(i) => i.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Real(v) => v.to_string(),
        Value::Nothing => "<nothing>".to_string(), // TODO: perhaps consider removing the Nothing type
    }
}
fn to_int(val: Value, span: SpanType) -> Result<i32, Box<RuntimeError>> {
    match val {
        Value::String(s) => match s.parse() {
            Ok(v) => Ok(v),
            Err(_) => Err(Box::new(RuntimeError {
                msg: format!("invalid `STRING` to convert to `INTEGER`: {s}"),
                span,
                ty: RuntimeErrorType::TypeError,
                info: vec![],
            })),
        },
        Value::Int(i) => Ok(i),
        t => Err(Box::new(RuntimeError {
            msg: format!("cannot convert a(n) {} to an `INTEGER`", t.ty()),
            span,
            ty: RuntimeErrorType::TypeError,
            info: vec![],
        })),
    }
}
fn to_bool(val: Value, span: SpanType) -> Result<bool, Box<RuntimeError>> {
    match val {
        Value::String(s) => Ok(matches!(s.to_lowercase().as_str(), "true" | "yes")),
        Value::Bool(v) => Ok(v),
        t => Err(Box::new(RuntimeError {
            msg: format!("cannot convert a(n) {} to a `BOOLEAN`", t.ty()),
            span,
            ty: RuntimeErrorType::TypeError,
            info: vec![],
        })),
    }
}

fn to_real(val: Value, span: SpanType) -> Result<NotNan<f64>, Box<RuntimeError>> {
    match val {
        Value::String(s) => match s.parse() {
            Ok(v) => Ok(v),
            Err(_) => Err(Box::new(RuntimeError {
                msg: format!("invalid `STRING` to convert to `REAL`: {s}"),
                span,
                ty: RuntimeErrorType::TypeError,
                info: vec![],
            })),
        },
        Value::Real(i) => Ok(i),
        Value::Int(i) => Ok(NotNan::new(i as f64).expect("ICE: float is NaN")),
        t => Err(Box::new(RuntimeError {
            msg: format!("cannot convert a(n) {} to a `REAL`", t.ty()),
            span,
            ty: RuntimeErrorType::TypeError,
            info: vec![],
        })),
    }
}

fn op_type_err(op: &str, t1: Type, t2: Type, span: SpanType) -> Box<RuntimeError> {
    Box::new(RuntimeError {
        msg: format!("cannot {op} a(n) {t1} and a(n) {t2}"),
        span,
        ty: RuntimeErrorType::TypeError,
        info: vec![],
    })
}
fn as_str(value: Value, span: SpanType) -> Result<String, Box<RuntimeError>> {
    match value {
        Value::String(v) => Ok(v),
        t => Err(Box::new(RuntimeError {
            msg: format!("expected type STRING, got type {}", t.ty()),
            span,
            ty: RuntimeErrorType::TypeError,
            info: vec![],
        })),
    }
}
fn example_type(ty: &Type) -> &str {
    match ty {
        Type::String => "\"example\"",
        Type::Int => "5",
        Type::Bool => "TRUE",
        Type::Char => "'a'",
        Type::Real => "3.14",
    }
}

fn divide(node: Value, operand: Value, span: SpanType) -> Result<Value, Box<RuntimeError>> {
    match node {
        Value::Int(i1) => match operand {
            Value::Int(i2) => {
                if i2 == 0 {
                    Err(Box::new(RuntimeError {
                        msg: String::from("cannot divide by 0"),
                        span,
                        ty: RuntimeErrorType::DivideByZeroError,
                        info: vec![],
                    }))
                } else {
                    Ok(Value::Real(
                        NotNan::new(i1 as f64 / i2 as f64).expect("ICE: float is NaN"),
                    ))
                }
            }
            Value::Real(r2) => {
                if r2 == 0.0 {
                    Err(Box::new(RuntimeError {
                        msg: String::from("cannot divide by 0"),
                        span,
                        ty: RuntimeErrorType::DivideByZeroError,
                        info: vec![],
                    }))
                } else {
                    Ok(Value::Real(
                        NotNan::new(i1 as f64 / *r2).expect("ICE: float is NaN"),
                    ))
                }
            }
            _ => Err(op_type_err("divide", node.ty(), operand.ty(), span)),
        },
        Value::Real(r1) => match operand {
            Value::Real(r2) => Ok(Value::Real(r1 / r2)),
            Value::Int(i2) => Ok(Value::Real(
                NotNan::new(*r1 / i2 as f64).expect("ICE: float is NaN"),
            )),
            _ => Err(op_type_err("divide", node.ty(), operand.ty(), span)),
        },
        _ => Err(op_type_err("divide", node.ty(), operand.ty(), span)),
    }
}
fn eq(node: &Value, operand: &Value) -> bool {
    match (node.clone(), operand.clone()) {
        (Value::Int(i), Value::Real(r)) | (Value::Real(r), Value::Int(i)) => i as f64 == *r,
        _ => node == operand,
    }
}
impl<I: InterpreterIO> Treewalker<I> {
    pub fn run(&mut self) -> Result<(), Box<RuntimeError>> {
        for node in self.nodes.clone() {
            // TODO: perhaps figure out how to not clone all the nodes
            self.exec_node(node)?;
        }
        Ok(())
    }
    pub fn new(nodes: &[ASTNode], io: I) -> Self {
        Self {
            nodes: nodes.to_vec(),
            variables: HashMap::new(),
            io,
            constants: HashMap::new(),
        }
    }

    fn set_var(&mut self, name: &str, val: Value, span: SpanType) -> Result<(), Box<RuntimeError>> {
        let var = match self.variables.get_mut(name) {
            None => {
                return Err(var_not_decl_err(name, span));
            }
            Some(v) => v,
        };
        if var.declared_ty != val.ty() {
            return Err(Box::new(RuntimeError {
                msg: format!(
                    "variable {name} declared as {}, but attempted to be set as {}",
                    var.declared_ty,
                    val.ty()
                ),
                span,
                ty: RuntimeErrorType::TypeError,
                info: vec![Info::help(format!(
                    "try changing the declared type to `{}`",
                    val.ty()
                ))],
            }));
        }
        var.value = Some(val);
        Ok(())
    }
    fn var_ty(&self, name: &str, span: SpanType) -> Result<Type, Box<RuntimeError>> {
        match self.variables.get(name) {
            Some(v) => Ok(v.declared_ty),
            None => Err(var_not_decl_err(name, span)),
        }
    }
    // fn expect_var_type(&self, name: &str, ty: &Type, span: Span) -> Result<&Value, RuntimeError> {
    //     let v = self.var_value(name, span.clone())?;
    //     if &v.ty() != ty {
    //         return Err(RuntimeError {
    //             msg: format!("expected type `{ty}`, got type `{}`", v.ty()),
    //             span,
    //             ty: RuntimeErrorType::TypeError,
    //             info: vec![],
    //         });
    //     }
    //     Ok(v)
    // }
    fn var_value(&self, name: &str, span: SpanType) -> Result<&Value, Box<RuntimeError>> {
        match self.variables.get(name) {
            Some(v) => match &v.value {
                Some(value) => Ok(value),
                None => Err(Box::new(RuntimeError {
                    msg: format!("variable {name} has not been set yet"),
                    span,
                    ty: RuntimeErrorType::ReferenceError,
                    info: vec![Info::help(format!(
                        "did you set the variable with (note that `{0}` is an example)\n    {name} <- {0}\nor did you input it with\n    INPUT {name}, \"optional prompt\"",
                        example_type(&v.declared_ty),
                    ))],
                })),
            },
            None => Err(var_not_decl_err(name, span)),
        }
    }
    fn exec_node(&mut self, astnode: ASTNode) -> Result<Value, Box<RuntimeError>> {
        match astnode.ty {
            ASTNodeType::If {
                condition,
                if_block,
                else_block,
            } => {
                let condition = match self.exec_node(ASTNode {
                    ty: ASTNodeType::Expr(condition.clone()),
                    span: SpanType::Syntax(condition.span.clone()),
                })? {
                    Value::Bool(b) => b,
                    t => {
                        return Err(Box::new(RuntimeError {
                            msg: format!("expected a BOOLEAN, got a `{}`", t.ty()),
                            span: SpanType::Syntax(condition.span),
                            ty: RuntimeErrorType::TypeError,
                            info: vec![],
                        }));
                    }
                };
                if condition {
                    for stmnt in if_block.0 {
                        self.exec_node(stmnt)?;
                    }
                } else if let Some(v) = else_block {
                    for stmnt in v.0 {
                        self.exec_node(stmnt)?;
                    }
                }
                Ok(Value::Nothing)
            }
            ASTNodeType::Input { ident, prompt } => {
                let span = SpanType::Syntax(ident.1);
                let ident = ident.0;
                self.var_ty(&ident, span.clone())?; // just to check that it exists before asking the prompt
                let p = match prompt {
                    Some(v) => as_str(
                        self.exec_node(ASTNode {
                            ty: ASTNodeType::Expr(v.clone()),
                            span: SpanType::Syntax(v.span.clone()),
                        })?,
                        SpanType::Syntax(v.span),
                    )?,
                    None => String::from("input> "),
                };
                self.io.print(&p);
                std::io::stdout().flush().unwrap();
                let mut buf = self.io.read_line(astnode.span.clone())?;
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                };
                let buf = Value::String(buf);

                let out = match self.var_ty(ident.as_str(), span.clone())? {
                    Type::String => buf,
                    Type::Int => to_int(buf, astnode.span)?.as_value(),
                    Type::Bool => to_bool(buf, astnode.span)?.as_value(),
                    Type::Char => todo!(),
                    Type::Real => to_real(buf, astnode.span)?.as_value(),
                };

                self.set_var(&ident, out, span)?;

                Ok(Value::Nothing)
            }
            ASTNodeType::Expr(expr) => match expr.ty {
                ExprType::Atom(atom) => match atom {
                    Atom::String(s) => Ok(Value::String(s)),
                    Atom::Int(i) => Ok(Value::Int(i)),
                    Atom::Ident(n) => {
                        if let Some(v) = self.constants.get(&n) {
                            return self.exec_node(ASTNode {
                                ty: ASTNodeType::Expr(Expr {
                                    ty: ExprType::Atom(v.value.0.clone()),
                                    span: v.value.1.clone(),
                                }),
                                span: SpanType::Syntax(v.value.1.clone()),
                            });
                        }
                        Ok(self.var_value(&n, SpanType::Syntax(expr.span))?.clone())
                    }
                    Atom::Real(r) => Ok(Value::Real(NotNan::new(r).expect("ICE: float is NaN"))),
                    Atom::Boolean(b) => Ok(Value::Bool(b)),
                },
                ExprType::BinOp(op, node, operand) => {
                    let node = self.exec_node(ASTNode {
                        ty: ASTNodeType::Expr(*node.clone()),
                        span: SpanType::Syntax(node.span),
                    })?;
                    let operand = self.exec_node(ASTNode {
                        ty: ASTNodeType::Expr(*operand.clone()),
                        span: SpanType::Syntax(operand.span),
                    })?;
                    match op {
                        BinOpType::Add => match node {
                            Value::Int(i1) => match operand {
                                Value::Int(v) => Ok(Value::Int(i1 + v)),
                                _ => Err(op_type_err("add", node.ty(), operand.ty(), astnode.span)),
                            },
                            Value::Real(r1) => match operand {
                                Value::Real(r2) => Ok(Value::Real(r1 + r2)),
                                Value::Int(i2) => Ok(Value::Real(
                                    NotNan::new(r1 + i2 as f64).expect("ICE: float is NaN"),
                                )),
                            _ => Err(op_type_err("add", node.ty(), operand.ty(), astnode.span)),
                            },
                            _ => Err(op_type_err("add", node.ty(), operand.ty(), astnode.span)),
                        },
                        BinOpType::Subtract => match node {
                            Value::Int(i1) => match operand {
                                Value::Int(i2) => Ok(Value::Int(i1 - i2)),
                                Value::Real(r2) => Ok(Value::Real(
                                    NotNan::new(i1 as f64 - *r2).expect("ICE: float is NaN"),
                                )),
                                _ => Err(op_type_err(
                                    "subtract",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            Value::Real(r1) => match operand {
                                Value::Real(r2) => Ok(Value::Real(r1 - r2)),
                                Value::Int(i2) => Ok(Value::Real(
                                    NotNan::new(r1 - i2 as f64).expect("ICE: float is NaN"),
                                )),
                                _ => Err(op_type_err(
                                    "subtract",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            _ => Err(op_type_err(
                                "subtract",
                                node.ty(),
                                operand.ty(),
                                astnode.span,
                            )),
                        },
                        BinOpType::Multiply => match node {
                            Value::Int(i1) => match operand {
                                Value::Int(i2) => Ok(Value::Int(i1 * i2)),
                                Value::Real(r2) => Ok(Value::Real(
                                    NotNan::new(i1 as f64 * *r2).expect("ICE: float is NaN"),
                                )),
                                _ => Err(op_type_err(
                                    "multiply",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            Value::Real(r1) => match operand {
                                Value::Real(r2) => Ok(Value::Real(r1 * r2)),
                                Value::Int(i2) => Ok(Value::Real(
                                    NotNan::new(*r1 * i2 as f64).expect("ICE: float is NaN"),
                                )),
                                _ => Err(op_type_err(
                                    "multiply",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            _ => Err(op_type_err(
                                "multiply",
                                node.ty(),
                                operand.ty(),
                                astnode.span,
                            )),
                        },
                        BinOpType::LT => match node {
                            Value::Int(i1) => match operand {
                                Value::Int(i2) => Ok(Value::Bool(i1 < i2)),
                                Value::Real(r2) => Ok(Value::Bool((i1 as f64) < *r2)),
                                _ => Err(op_type_err(
                                    "check if less than",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            Value::Real(r1) => match operand {
                                Value::Real(r2) => Ok(Value::Bool(r1 < r2)),
                                Value::Int(i2) => Ok(Value::Bool(r1 < i2.into())),
                                _ => Err(op_type_err(
                                    "check if less than",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            _ => Err(op_type_err(
                                "check if less than",
                                node.ty(),
                                operand.ty(),
                                astnode.span,
                            )),
                        },
                        BinOpType::LTE => match node {
                            Value::Int(i1) => match operand {
                                Value::Int(i2) => Ok(Value::Bool(i1 <= i2)),
                                Value::Real(r2) => Ok(Value::Bool((i1 as f64) <= *r2)),
                                _ => Err(op_type_err(
                                    "check if less than or equal to",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            Value::Real(r1) => match operand {
                                Value::Real(r2) => Ok(Value::Bool(r1 <= r2)),
                                Value::Int(i2) => Ok(Value::Bool(r1 <= i2.into())),
                                _ => Err(op_type_err(
                                    "check if less than or equal to",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            _ => Err(op_type_err(
                                "check if less than or equal to",
                                node.ty(),
                                operand.ty(),
                                astnode.span,
                            )),
                        },
                        BinOpType::GT => match node {
                            Value::Int(i1) => match operand {
                                Value::Int(i2) => Ok(Value::Bool(i1 > i2)),
                                Value::Real(r2) => Ok(Value::Bool((i1 as f64) > *r2)),
                                _ => Err(op_type_err(
                                    "check if greater than",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            Value::Real(r1) => match operand {
                                Value::Real(r2) => Ok(Value::Bool(r1 > r2)),
                                Value::Int(i2) => Ok(Value::Bool(r1 < i2.into())),
                                _ => Err(op_type_err(
                                    "check if greater than",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            _ => Err(op_type_err(
                                "check if greater than",
                                node.ty(),
                                operand.ty(),
                                astnode.span,
                            )),
                        },
                        BinOpType::GTE => match node {
                            Value::Int(i1) => match operand {
                                Value::Int(i2) => Ok(Value::Bool(i1 >= i2)),
                                Value::Real(r2) => Ok(Value::Bool((i1 as f64) >= *r2)),
                                _ => Err(op_type_err(
                                    "check if greater than or equal to",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            Value::Real(r1) => match operand {
                                Value::Real(r2) => Ok(Value::Bool(r1 >= r2)),
                                Value::Int(i2) => Ok(Value::Bool(r1 >= i2.into())),
                                _ => Err(op_type_err(
                                    "check if greater than or equal to",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            _ => Err(op_type_err(
                                "check if greater than or equal to",
                                node.ty(),
                                operand.ty(),
                                astnode.span,
                            )),
                        },
                        BinOpType::IntegerDivide => match divide(node, operand, astnode.span)? {
                            Value::Real(r) => Ok(Value::Int(r.floor() as i32)),
                            _ => unreachable!(),
                        },
                        BinOpType::Divide => divide(node, operand, astnode.span),
                        BinOpType::Modulo => match node {
                            Value::Int(i1) => match operand {
                                Value::Int(i2) => {
                                    if i2 == 0 {
                                        Err(Box::new(RuntimeError {
                                            msg: String::from("cannot modulo by 0"),
                                            span: astnode.span,
                                            ty: RuntimeErrorType::DivideByZeroError,
                                            info: vec![Info::note(
                                                "modulo gives the remainder of division, so you cannot modulo by 0",
                                            )],
                                        }))
                                    } else {
                                        Ok(Value::Int(i1 % i2))
                                    }
                                }
                                Value::Real(r2) => {
                                    if r2 == 0.0 {
                                        Err(Box::new(RuntimeError {
                                            msg: String::from("cannot modulo by 0"),
                                            span: astnode.span,
                                            ty: RuntimeErrorType::DivideByZeroError,
                                            info: vec![Info::note(
                                                "modulo gives the remainder of division, so you cannot modulo by 0",
                                            )],
                                        }))
                                    } else {
                                        Ok(Value::Real(
                                            NotNan::new(i1 as f64 % *r2)
                                                .expect("ICE: float is NaN"),
                                        ))
                                    }
                                }
                                _ => Err(op_type_err(
                                    "modulo",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            Value::Real(r1) => match operand {
                                Value::Real(r2) => Ok(Value::Real(r1 % r2)),
                                Value::Int(i2) => Ok(Value::Real(
                                    NotNan::new(*r1 % i2 as f64).expect("ICE: float is NaN"),
                                )),
                                _ => Err(op_type_err(
                                    "modulo",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            _ => Err(op_type_err("modulo", node.ty(), operand.ty(), astnode.span)),
                        },

                        BinOpType::LogicalOr => match node {
                            Value::Bool(b1) => match operand {
                                Value::Bool(b2) => Ok(Value::Bool(b1 || b2)),
                                _ => Err(op_type_err(
                                    "logical OR",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            _ => Err(op_type_err(
                                "logical OR",
                                node.ty(),
                                operand.ty(),
                                astnode.span,
                            )),
                        },
                        BinOpType::LogicalAnd => match node {
                            Value::Bool(b1) => match operand {
                                Value::Bool(b2) => Ok(Value::Bool(b1 && b2)),
                                _ => Err(op_type_err(
                                    "logical AND",
                                    node.ty(),
                                    operand.ty(),
                                    astnode.span,
                                )),
                            },
                            _ => Err(op_type_err(
                                "logical AND",
                                node.ty(),
                                operand.ty(),
                                astnode.span,
                            )),
                        },
                        BinOpType::Equality => Ok(Value::Bool(eq(&node, &operand))),
                        BinOpType::InEquality => Ok(Value::Bool(!eq(&node, &operand))),
                    }
                }
                ExprType::UnaryOp(op, node) => {
                    let node = self.exec_node(ASTNode {
                        ty: ASTNodeType::Expr(*node.clone()),
                        span: SpanType::Syntax(node.span),
                    })?;
                    match op {
                        UnaryOpType::LogicalNot => Ok(Value::Bool(!to_bool(node, astnode.span)?)),
                        UnaryOpType::Negate => Ok(Value::Int(-to_int(node, astnode.span)?)),
                        UnaryOpType::Positive => Ok(Value::Int(to_int(node, astnode.span)?)),
                    }
                }
            },
            ASTNodeType::Output(exprs) => {
                let mut out = Vec::new();
                for expr in exprs {
                    out.push(to_str(self.exec_node(ASTNode {
                        ty: ASTNodeType::Expr(expr.clone()),
                        span: SpanType::Syntax(expr.span),
                    })?))
                }
                self.io.print(&format!("{}\n", out.join(" ")));
                Ok(Value::Nothing)
            }
            ASTNodeType::Declare {
                ident: name,
                ty: declared_ty,
            } => {
                self.variables.insert(
                    name.clone(),
                    Variable {
                        name,
                        declared_ty,
                        value: None,
                    },
                );
                Ok(Value::Nothing)
            }
            ASTNodeType::Case {
                value,
                cases,
                otherwise,
            } => {
                let value = self.exec_node(ASTNode {
                    ty: ASTNodeType::Expr(value.clone()),
                    span: SpanType::Syntax(value.span),
                })?;
                let mut ran = false;
                for (case, stmnts) in cases {
                    let case = self.exec_node(ASTNode {
                        ty: ASTNodeType::Expr(Expr {
                            ty: ExprType::Atom(case.0),
                            span: case.1.clone(),
                        }),
                        span: SpanType::Syntax(case.1),
                    })?;
                    if eq(&value, &case) {
                        ran = true;
                        for stmnt in stmnts {
                            self.exec_node(stmnt)?;
                        }
                    }
                }
                if let Some(v) = otherwise
                    && !ran
                {
                    for stmnt in v {
                        self.exec_node(stmnt)?;
                    }
                }
                Ok(Value::Nothing)
            }
            ASTNodeType::Assign { ident: i, value } => {
                let ident = i.0;
                let ident_span = i.1;
                self.var_ty(&ident, SpanType::Syntax(ident_span.clone()))?; // check if the variable is declared
                let val = self.exec_node(ASTNode {
                    ty: ASTNodeType::Expr(value.clone()),
                    span: SpanType::Syntax(value.span),
                })?;
                self.set_var(&ident, val, astnode.span)?;

                Ok(Value::Nothing)
            }
            ASTNodeType::Constant { ident, value } => {
                self.constants.insert(ident.clone().0, Constant { value });
                Ok(Value::Nothing)
            }
        }
    }
}
