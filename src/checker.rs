use std::collections::HashMap;

use crate::{
    parser::{ASTNode, ASTNodeType, Atom, BinOpType, Expr, ExprType, SpanType, UnaryOpType},
    treewalker::Type,
    types::{DiagType, Diagnostic, ErrType, Info, Span, Stage},
};

#[derive(Clone, Copy, PartialEq)]
pub enum VarType {
    Const,
    Variable,
}
pub struct Checker {
    declared_vars: HashMap<String, (Type, VarType)>,
    auto_decl: Option<Box<dyn FnMut(String) -> Result<Type, Diagnostic>>>,
    pub generated_decls: Vec<(String, Type)>,
}

pub struct DeclResolution {
    ty: Type,
    name: String,
}
fn type_error(expected: Vec<Type>, got: Type, span: Span) -> Diagnostic {
    type_error_info(expected, got, span, vec![])
}
fn type_error_info(expected: Vec<Type>, got: Type, span: Span, info: Vec<Info>) -> Diagnostic {
    Diagnostic {
        ty: DiagType::Err(ErrType::TypeError { expected, got }, Stage::Checker),
        info,
        span: Some(SpanType::Syntax(span)),
    }
}

#[inline]
pub fn undeclared_variable_info(name: String, ty: Option<Type>) -> Vec<Info> {
    vec![
        Info::help(format!(
            "did you declare it with\n    DECLARE {name} : {}",
            ty.map(|t| t.to_string()).unwrap_or(String::from("// type"))
        )),
        Info::help(
            "does your syllabus cover `DECLARE`? if not, run with the `--interactive` or `--interactive --rewrite` flag ",
        ),
    ]
}

impl Checker {
    pub fn new(auto_decl: Option<Box<dyn FnMut(String) -> Result<Type, Diagnostic>>>) -> Self {
        Self {
            declared_vars: HashMap::new(),
            auto_decl,
            generated_decls: Vec::new(),
        }
    }
    pub fn check_all(&mut self, nodes: Vec<ASTNode>) -> Result<(), Diagnostic> {
        for node in nodes {
            self.check_node(&node)?;
        }
        Ok(())
    }
    fn expect_type(&self, e: &Expr, ty: Vec<Type>) -> Result<Type, Diagnostic> {
        let t = self.infer_type(e)?;
        if !ty.contains(&t) {
            Err(Diagnostic {
                ty: DiagType::Err(
                    ErrType::TypeError {
                        expected: ty,
                        got: t,
                    },
                    Stage::Checker,
                ),
                info: vec![],
                span: Some(SpanType::Syntax(e.span.clone())),
            })
        } else {
            Ok(t)
        }
    }
    pub fn infer_type(&self, e: &Expr) -> Result<Type, Diagnostic> {
        match &e.ty {
            ExprType::Atom(atom) => match atom.ty() {
                Some(v) => Ok(v),
                None => match atom {
                    Atom::Ident(a) => {
                        if let Some((t, _)) = self.declared_vars.get(a) {
                            Ok(t.to_owned())
                        } else {
                            Err(Diagnostic {
                                ty: DiagType::Err(
                                    ErrType::UndeclaredVariable {
                                        ident: a.to_owned(),
                                    },
                                    Stage::Checker,
                                ),
                                info: undeclared_variable_info(a.to_owned(), None),
                                span: Some(SpanType::Syntax(e.span.clone())),
                            })
                        }
                    }
                    _ => unreachable!(),
                },
            },
            ExprType::UnaryOp(unary_op_type, expr) => match unary_op_type {
                UnaryOpType::LogicalNot => {
                    self.expect_type(expr, vec![Type::Bool])?;
                    Ok(Type::Bool)
                }
                UnaryOpType::Negate => {
                    let t = self.expect_type(expr, vec![Type::Real, Type::Int])?;
                    Ok(t)
                }
                UnaryOpType::Positive => {
                    let t = self.expect_type(expr, vec![Type::Real, Type::Int])?;
                    Ok(t)
                }
            },
            ExprType::BinOp(bin_op_type, expr, expr1) => {
                let expr_t = self.infer_type(expr)?;
                let expr1_t = self.infer_type(expr1)?;
                match bin_op_type {
                    // TODO: fix duplication
                    BinOpType::Add => match expr_t {
                        Type::Int => match expr1_t {
                            Type::Int => Ok(Type::Int),
                            Type::Real => Ok(Type::Real),
                            t => Err(type_error(
                                vec![Type::Int, Type::Real],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Real),
                            t => Err(type_error(
                                vec![Type::Real, Type::Int],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                    BinOpType::Subtract => match expr_t {
                        Type::Int => match expr1_t {
                            Type::Int => Ok(Type::Int),
                            Type::Real => Ok(Type::Real),
                            t => Err(type_error(
                                vec![Type::Int, Type::Real],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Real),
                            t => Err(type_error(
                                vec![Type::Real, Type::Int],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                    BinOpType::Multiply => match expr_t {
                        Type::Int => match expr1_t {
                            Type::Int => Ok(Type::Int),
                            Type::Real => Ok(Type::Real),
                            t => Err(type_error(
                                vec![Type::Int, Type::Real],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Real),
                            t => Err(type_error(
                                vec![Type::Real, Type::Int],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                    BinOpType::Divide => match expr_t {
                        Type::Int => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Real),
                            t => Err(type_error(
                                vec![Type::Int, Type::Real],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Real),
                            t => Err(type_error(
                                vec![Type::Real, Type::Int],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                    BinOpType::Modulo => match expr_t {
                        Type::Int => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Real),
                            t => Err(type_error(
                                vec![Type::Int, Type::Int],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Int),
                            t => Err(type_error(
                                vec![Type::Real, Type::Int],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                    BinOpType::LogicalOr => match expr_t {
                        Type::Bool => match expr1_t {
                            Type::Bool => Ok(Type::Bool),
                            t => Err(type_error(vec![Type::Bool], t, expr1.span.clone())),
                        },
                        t => Err(type_error(vec![Type::Bool], t, expr.span.clone())),
                    },
                    BinOpType::LogicalAnd => match expr_t {
                        Type::Bool => match expr1_t {
                            Type::Bool => Ok(Type::Bool),
                            t => Err(type_error(vec![Type::Bool], t, expr1.span.clone())),
                        },
                        t => Err(type_error(vec![Type::Bool], t, expr.span.clone())),
                    },
                    BinOpType::Equality => {
                        if expr_t == expr1_t
                            || matches!(
                                (expr_t, expr1_t),
                                (Type::Real | Type::Int, Type::Real | Type::Int)
                            )
                        {
                            Ok(Type::Bool)
                        } else {
                            Err(type_error_info(
                                vec![expr_t],
                                expr1_t,
                                e.span.clone(),
                                vec![Info::note(
                                    "both sides must be the same type, or both sides must be a number type.",
                                )],
                            ))
                        }
                    }
                    BinOpType::InEquality => {
                        if expr_t == expr1_t
                            || matches!(
                                (expr_t, expr1_t),
                                (Type::Real | Type::Int, Type::Real | Type::Int)
                            )
                        {
                            Ok(Type::Bool)
                        } else {
                            Err(type_error_info(
                                vec![expr_t],
                                expr1_t,
                                e.span.clone(),
                                vec![Info::note(
                                    "both sides must be the same type, or both sides must be a number type.",
                                )],
                            ))
                        }
                    }
                    BinOpType::LT => match expr_t {
                        Type::Int | Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Bool),
                            t => Err(type_error(
                                vec![Type::Int, Type::Real],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                    BinOpType::LTE => match expr_t {
                        Type::Int | Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Bool),
                            t => Err(type_error(
                                vec![Type::Int, Type::Real],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                    BinOpType::GT => match expr_t {
                        Type::Int | Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Bool),
                            t => Err(type_error(
                                vec![Type::Int, Type::Real],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                    BinOpType::GTE => match expr_t {
                        Type::Int | Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Bool),
                            t => Err(type_error(
                                vec![Type::Int, Type::Real],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                    BinOpType::IntegerDivide => match expr_t {
                        Type::Int | Type::Real => match expr1_t {
                            Type::Int | Type::Real => Ok(Type::Int),
                            t => Err(type_error(
                                vec![Type::Int, Type::Real],
                                t,
                                expr1.span.clone(),
                            )),
                        },
                        t => Err(type_error(
                            vec![Type::Int, Type::Real],
                            t,
                            expr.span.clone(),
                        )),
                    },
                }
            }
        }
    }
    fn handle_undeclared(
        &mut self,
        ident: String,
        span: Span,
        ty: Option<Type>,
    ) -> Result<(), Diagnostic> {
        if let Some(ref mut for_decl) = self.auto_decl {
            let t = for_decl(ident.clone())?;
            self.generated_decls.push((ident.to_owned(), t));
            self.declared_vars.insert(ident.clone(), (t, VarType::Variable));
            return Ok(());
        }
        return Err(Diagnostic {
            ty: DiagType::Err(
                ErrType::UndeclaredVariable {
                    ident: ident.clone(),
                },
                Stage::Checker,
            ),
            info: undeclared_variable_info(ident.clone(), ty),
            span: Some(SpanType::Syntax(span.clone())),
        });
    }
    pub fn check_node(&mut self, node: &ASTNode) -> Result<(), Diagnostic> {
        match node.ty {
            ASTNodeType::Declare { ref ident, ty } => {
                if self.declared_vars.contains_key(ident) {
                    return Err(Diagnostic {
                        ty: DiagType::Err(
                            ErrType::DuplicateDeclare {
                                ident: ident.to_owned(),
                            },
                            Stage::Checker,
                        ),
                        info: vec![Info::help("you declared this variable twice")],
                        span: Some(node.span.clone()),
                    });
                }
                self.declared_vars
                    .insert(ident.to_owned(), (ty, VarType::Variable));
                Ok(())
            }
            ASTNodeType::Assign {
                ref ident,
                ref value,
            } => {
                let ty = match self.declared_vars.get(&ident.0) {
                    None => return self.handle_undeclared(ident.0.clone(), ident.1.clone(), Some(self.infer_type(value)?)),
                    Some((v, t)) => match t {
                        VarType::Const => {
                            return Err(Diagnostic {
                                ty: DiagType::Err(
                                    ErrType::AttemptedModifyingConst {
                                        ident: ident.0.to_owned(),
                                    },
                                    Stage::Checker,
                                ),
                                info: vec![Info::note("you cannot modify constants")],
                                span: Some(node.span.clone()),
                            });
                        }
                        VarType::Variable => v,
                    },
                };
                self.expect_type(value, vec![*ty])?;
                Ok(())
            }
            ASTNodeType::Expr(ref e) => {
                self.infer_type(e)?;
                Ok(())
            }
            ASTNodeType::If { ref condition, .. } => {
                self.expect_type(condition, vec![Type::Bool])?;
                Ok(())
            }
            ASTNodeType::Input {
                ref prompt,
                ref ident,
            } => {
                if let Some(v) = prompt {
                    self.infer_type(v)?;
                }
                if let Some(v) = self.declared_vars.get(&ident.0) {
                    if v.1 == VarType::Const {
                        return Err(Diagnostic {
                            ty: DiagType::Err(
                                ErrType::AttemptedModifyingConst {
                                    ident: ident.0.clone(),
                                },
                                Stage::Checker,
                            ),
                            info: vec![Info::note("you cannot INPUT a value into a constant")],
                            span: Some(SpanType::Syntax(ident.1.clone())),
                        });
                    }
                } else {
                    return self.handle_undeclared(ident.0.clone(), ident.1.clone(), None);
                }
                Ok(())
            }
            ASTNodeType::Case {
                ref value,
                ref cases,
                ..
            } => {
                let ty = self.infer_type(value)?;
                let t = if let Type::Int | Type::Real = ty {
                    vec![Type::Int, Type::Real]
                } else {
                    vec![ty]
                };
                for ((atom, span), _) in cases {
                    let got = atom.ty().expect("got non static atom in CASE");
                    if !t.contains(&got) {
                        return Err(Diagnostic {
                            ty: DiagType::Err(
                                ErrType::TypeError { expected: t, got },
                                Stage::Checker,
                            ),
                            info: vec![Info::help(format!(
                                "this case statement is against {}, which is of type {}",
                                value, ty
                            ))],
                            span: Some(SpanType::Syntax(span.clone())),
                        });
                    }
                }
                Ok(())
            }
            ASTNodeType::Constant {
                ref ident,
                ref value,
            } => {
                let t = value.0.ty().expect("constant with non-static value");
                if self.declared_vars.contains_key(&ident.0) {
                    return Err(Diagnostic {
                        ty: DiagType::Err(
                            ErrType::DuplicateDeclare {
                                ident: ident.0.to_owned(),
                            },
                            Stage::Checker,
                        ),
                        info: vec![],
                        span: Some(node.span.clone()),
                    });
                }
                self.declared_vars
                    .insert(ident.0.clone(), (t, VarType::Const));
                Ok(())
            }
            ASTNodeType::Output(ref exprs) => {
                for e in exprs {
                    self.infer_type(e)?;
                }
                Ok(())
            }
        }
    }
}
