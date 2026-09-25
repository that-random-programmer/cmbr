use std::{fmt::Display, rc::Rc};

use crate::{
    tokenizer::{TokenKind, TokenType},
    treewalker::RuntimeError,
};
#[derive(Debug, Clone)]
pub struct Span {
    pub ln: usize,
    pub col: usize,
    pub pos: usize,
    pub endln: Option<usize>,
    pub endcol: Option<usize>,
    pub endpos: Option<usize>,
    pub fp: Rc<str>,
}

impl Span {
    pub fn merge(&self, other: Self) -> Self {
        match other.endln {
            Some(_) => Self {
                ln: self.ln,
                col: self.col,
                pos: self.pos,
                endln: other.endln,
                endcol: other.endcol,
                endpos: other.endpos,
                fp: self.fp.clone(),
            },
            None => Self {
                ln: self.ln,
                col: self.col,
                pos: self.pos,
                endln: Some(other.ln),
                endcol: Some(other.col),
                endpos: Some(other.pos),
                fp: self.fp.clone(),
            },
        }
    }
    pub fn empty(fp: impl Into<Rc<str>>) -> Self {
        Span {
            ln: 0,
            col: 0,
            pos: 0,
            endln: None,
            endcol: None,
            endpos: None,
            fp: fp.into(),
        }
    }
}
#[derive(Debug, Clone)]
pub enum ErrType {
    InvalidChar {
        char: Option<char>,
    },
    ExpectedNewline,
    UnexpectedEOF,
    UnterminatedStringLiteral,
    UnexpectedToken {
        tkn: TokenType,
        expected: Option<Vec<TokenKind>>,
    },
}

#[derive(Debug, Clone)]
pub enum WarnType {}

impl Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrType::InvalidChar { char: ch } => match ch {
                Some(v) => write!(f, "invalid char: `{v}`"),
                None => write!(f, "invalid char"),
            },

            ErrType::UnexpectedEOF => write!(f, "unexpected EOF"),
            ErrType::UnterminatedStringLiteral => write!(f, "unterminated string literal"),
            ErrType::UnexpectedToken { tkn, expected } => match expected {
                Some(e) if e.len() != 0 => {
                    if e.len() == 1 {
                        write!(f, "expected token `{}`, got `{tkn}`", e[0])
                    } else if e.len() == 2 {
                        write!(f, "expected token `{}` or `{}`, got `{tkn}`", e[0], e[1])
                    } else {
                        write!(f, "expected token ")?;
                        for i in &e[..e.len() - 1] {
                            write!(f, "`{i}`, ")?;
                        }
                        write!(f, "or `{}`, got `{tkn}`", e.last().unwrap())
                    }
                }
                None | Some(_) => write!(f, "unexpected token `{tkn}`"),
            },
            ErrType::ExpectedNewline => write!(f, "expected a newline after this statement"),
        }
    }
}
impl Display for DiagType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagType::Err(err_type) => write!(f, "{err_type}"),
            DiagType::Warn(warn_type) => match warn_type {
                _ => todo!(),
            },
        }
    }
}


#[derive(Debug, Clone)]
pub enum DiagType {
    Err(ErrType),
    Warn(WarnType), // TODO: warnings via iterator
}
pub trait Failure {
    fn span(&self) -> Option<&Span>;
    fn msg(&self) -> String;
    fn info(&self) -> &Vec<Info>;
}

impl Failure for RuntimeError {
    fn span(&self) -> Option<&Span> {
        Some(&self.span)
    }

    fn msg(&self) -> String {
        format!("{} (runtime error): {}", self.ty, self.msg.clone())
    }

    fn info(&self) -> &Vec<Info> {
        &self.info
    }
}

impl Failure for Diagnostic {
    fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }

    fn msg(&self) -> String {
        format!("{}", self.ty)
    }

    fn info(&self) -> &Vec<Info> {
        &self.info
    }
}
impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.fp, self.ln + 1, self.col + 1)?;
        if let Some(ln) = self.endln {
            write!(f, " until {}", ln + 1)?;
            if let Some(col) = self.endcol {
                write!(f, ":{}", col + 1)?;
            }
        }
        Ok(())
    }
}
impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)?;
        if let Some(v) = &self.span {
            write!(f, "\nat {}:{}:{}", v.fp, v.ln + 1, v.col + 1)?;
            if let Some(ln) = v.endln {
                write!(f, " until {}", ln + 1)?;
                if let Some(col) = v.endcol {
                    write!(f, ":{}", col + 1)?;
                }
            }
        }
        for i in &self.info {
            write!(f, "\n{i}")?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub ty: DiagType,
    pub info: Vec<Info>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub msg: String,
    pub ty: InfoType,
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.ty, self.msg)
    }
}
impl Display for InfoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfoType::Help => write!(f, "help"),
            InfoType::Note => write!(f, "note"),
        }
    }
}

impl Info {
    pub fn help(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            ty: InfoType::Help,
        }
    }

    pub fn note(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            ty: InfoType::Note,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InfoType {
    Help,
    Note,
}
