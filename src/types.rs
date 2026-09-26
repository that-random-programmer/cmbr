use std::{fmt::Display, rc::Rc};

use owo_colors::OwoColorize;

use crate::{
    tokenizer::{TokenKind, TokenType},
    treewalker::{RuntimeError, RuntimeErrorType},
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

pub struct DiagnosticPrinter<I: InterpreterIO> {
    file_content: Rc<str>,
    io: I,
}

impl<I: InterpreterIO> DiagnosticPrinter<I> {
    pub fn new(file_content: impl Into<Rc<str>>, io: I) -> Self {
        Self {
            file_content: file_content.into(),
            io,
        }
    }
}

pub struct CLIInterpreterIO;
impl InterpreterIO for CLIInterpreterIO {
    fn print(&self, s: &str) {
        print!("{s}");
    }

    fn read_line(&self, span: Span) -> Result<String, RuntimeError> {
        let mut buf = String::new();
        match std::io::stdin().read_line(&mut buf) {
            Ok(_) => Ok(buf),
            Err(e) => Err(RuntimeError {
                msg: format!("failed to read from stdin: {e}"),
                span: span,
                ty: RuntimeErrorType::IOError,
                info: vec![Info::note(
                    "this error is not caused by something wrong with your program",
                )],
            }),
        }
    }
}

impl<I: InterpreterIO> DiagnosticPrinter<I> {
    pub fn print_diagnostic(&self, diag: &impl Failure) {
        self.io.println(&format!(
            "{}{}",
            "error".red().bold(),
            format!(": {}", diag.msg().bold())
        ));

        if let Some(span) = &diag.span() {
            self.io.println(&format!("at {span}"));
            let start = span.ln.saturating_sub(3);
            let end = match span.endln {
                Some(v) => v + 3,
                None => span.ln + 3,
            }
            .clamp(0, self.file_content.lines().count());
            let line_count = end - start;
            let mut arrow_drawn = false;
            let no_endln = match span.endln {
                Some(v) => v == span.ln,
                None => true,
            };
            for (ln, i) in self
                .file_content
                .lines()
                .skip(start)
                .take(line_count)
                .zip(start..end)
            {
                let start;
                if let Some(endln) = span.endln
                    && !no_endln
                    && span.ln < i
                    && i <= endln
                    && !arrow_drawn
                {
                    start = "-> ";
                    arrow_drawn = true;
                } else if span.ln == i {
                    start = "-> ";
                    arrow_drawn = true;
                } else {
                    start = "   "
                }
                if span.endln.is_none()
                    || span.endln.is_some_and(|endln| !(endln > i && i > span.ln))
                {
                    self.io.println(&format!(
                        "{}{}{}",
                        start.red().bold(),
                        format!("{:4} | ", i + 1).blue().bold(),
                        ln
                    ));
                }
                if let Some(endln) = span.endln
                    && span.ln <= i
                    && i <= endln
                    && let Some(endcol) = span.endcol
                    && !no_endln
                {
                    if i == span.ln {
                        self.io.println(&format!(
                            "        {} {}{} {}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".repeat(ln.chars().count() - span.col + 1).red().bold(),
                            "from here".purple().bold()
                        ));
                    } else if i == span.ln + 1 {
                        self.io.println(&format!("        {}", "| ...".blue().bold()))
                    } else if i == endln {
                        self.io.println(&format!(
                            "        {} {}{} {}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".repeat(endcol).red().bold(),
                            "to here".purple().bold()
                        ));
                    }
                } else if span.ln == i && no_endln {
                    if let Some(endcol) = span.endcol {
                        self.io.println(&format!(
                            "        {} {}{}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".repeat(endcol - span.col + 1).red().bold()
                        ))
                    } else {
                        self.io.println(&format!(
                            "        {} {}{}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".red().bold()
                        ))
                    }
                }
            }
            self.io.println("")
        }
        for info in diag.info() {
            self.io.println(&format!(
                "{}{}",
                info.ty.bold().blue(),
                format!(": {}", info.msg)
            ));
        }
    }
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

pub trait InterpreterIO {
    fn read_line(&self, span: Span) -> Result<String, RuntimeError>;
    fn println(&self, s: &str) {
        self.print(&format!("{s}\n"));
    }
    fn print(&self, s: &str);
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
