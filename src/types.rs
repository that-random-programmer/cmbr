use std::{fmt::Display, rc::Rc};

use owo_colors::{AnsiColors, OwoColorize};

use crate::{
    parser::SpanType, tokenizer::{TokenKind, TokenType}, treewalker::{RuntimeError, RuntimeErrorType, Type},
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

    fn read_line(&self, span: SpanType) -> Result<String, RuntimeError> {
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
        let color = if diag.is_critical() {
            AnsiColors::Red
        } else {
            AnsiColors::Yellow
        };

        self.io.println(&format!(
            "{}{}",
            diag.ty().to_string().color(color).bold(),
            format!(": {}", diag.msg().bold())
        ));

        if let Some(SpanType::Syntax(span)) = &diag.span() {
            self.io.println(&format!("at {span}"));
            let start = span.ln.saturating_sub(3);
            let end = match span.endln {
                Some(v) => v + 3,
                None => span.ln + 3,
            }
            .clamp(0, self.file_content.lines().count());
            let line_count = end - start;
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
                    && i < endln
                {
                    start = "|  ";
                } else if let Some(endln) = span.endln
                    && !no_endln
                    && (i == span.ln || i == endln)
                {
                    start = "+--"
                } else if span.ln == i {
                    start = "-> ";
                } else {
                    start = "   "
                }
                if span.endln.is_none()
                    || span.endln.is_some_and(|endln| {
                        !(endln > i && i > span.ln)
                            || (span.ln < i && i < endln && matches!(endln - span.ln, 2..=3))
                    })
                {
                    self.io.println(&format!(
                        "{}{}{}",
                        start.color(color).bold(),
                        format!("{}{:4} | ", "", i + 1).blue().bold(),
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
                            "{}       {} {}{} {}",
                            "|".color(color).bold(),
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".color(color).bold(),
                            "from here...".purple().bold()
                        ));
                    } else if i == span.ln + 1 && !matches!(endln - span.ln, 1..=3) {
                        self.io.println(&format!(
                            "{}       {}",
                            "|".color(color).bold(),
                            "| ...".blue().bold()
                        ))
                    } else if i == endln {
                        self.io.println(&format!(
                            "        {} {}{} {}",
                            "|".blue().bold(),
                            " ".repeat(endcol),
                            "^".color(color).bold(),
                            "to here".purple().bold()
                        ));
                    }
                } else if span.ln == i && no_endln {
                    if let Some(endcol) = span.endcol {
                        self.io.println(&format!(
                            "        {} {}{}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".repeat(endcol - span.col + 1).color(color).bold()
                        ))
                    } else {
                        self.io.println(&format!(
                            "        {} {}{}",
                            "|".blue().bold(),
                            " ".repeat(span.col),
                            "^".color(color).bold()
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
                format!(": {}", info.msg.replace("\n", "\n      "))
            ));
        }
        self.io.println("")
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
    pub fn eof(from: Span) -> Self {
        Span {
            ln: from.endln.unwrap_or(from.ln),
            col: from.endcol.unwrap_or(from.col),
            pos: from.endpos.unwrap_or(from.pos),
            endln: None,
            endcol: None,
            endpos: None,
            ..from
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
    DuplicateDeclare {
        ident: String,
    },
    UndeclaredVariable {
        ident: String,
    },
    TypeError {
        expected: Vec<Type>,
        got: Type,
    },
    AttemptedModifyingConst {
        ident: String,
    },
    Other {
        msg: String
    }
}

#[derive(Debug, Clone)]
pub enum WarnType {
    InputWithPrompt,
    UnconventionalVariableName,
    AmbiguousVariableName,
}

impl Display for WarnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarnType::InputWithPrompt => write!(f, "input statement with a prompt"),
            WarnType::UnconventionalVariableName => write!(f, "unconventional variable name"),
            WarnType::AmbiguousVariableName => write!(f, "ambiguous variable name"),
        }
    }
}

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
                Some(e) if e.is_empty() => {
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
            ErrType::DuplicateDeclare { ident: name } => {
                write!(f, "duplicate declaration for {name}")
            }
            ErrType::UndeclaredVariable { ident: name } => {
                write!(f, "variable {name} never declared")
            }
            ErrType::TypeError { expected, got } => {
                if expected.is_empty() {
                    write!(f, "unexpected type {got}")
                } else if expected.len() == 1 {
                    write!(f, "expected type {}, got {got}", expected[0])
                }else if expected.len() == 2 {
                    write!(
                        f,
                        "expected type {} or {}, got {got}",
                        expected[0], expected[1]
                    )
                } else {
                    write!(f, "expected type ")?;
                    for t in &expected[..expected.len() - 1] {
                        write!(f, "{t}, ")?;
                    }
                    write!(f, "or {}, got `{got}`", expected.last().unwrap())
                }
            }
            ErrType::AttemptedModifyingConst { ident } => {
                write!(f, "attempted to modify constant {ident}")
            },
            ErrType::Other { msg } => write!(f, "{msg}")
        }
    }
}
impl Display for DiagType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagType::Err(err_type, _) => write!(f, "{err_type}"),
            DiagType::Warn(warn_type) => write!(f, "{warn_type}"),
        }
    }
}

pub trait InterpreterIO {
    fn read_line(&self, span: SpanType) -> Result<String, RuntimeError>;
    fn println(&self, s: &str) {
        self.print(&format!("{s}\n"));
    }
    fn print(&self, s: &str);
}

#[derive(Clone, Debug, Copy)]
pub enum Stage {
    Tokenizer,
    Parser,
    Checker,
}
#[derive(Debug, Clone)]
pub enum DiagType {
    Err(ErrType, Stage),
    Warn(WarnType),
}

#[derive(Clone, Debug, Copy)]
pub enum FailureType {
    ParseError,
    TokenizerError,
    RuntimeError,
    Warning,
    CheckerError,
}
impl Display for FailureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureType::ParseError => write!(f, "error[parsing]"),
            FailureType::Warning => write!(f, "warning"),
            FailureType::RuntimeError => write!(f, "error[runtime]"),
            FailureType::TokenizerError => write!(f, "error[lex]"), // lex is more clear
            FailureType::CheckerError => write!(f, "error[check]"),
        }
    }
}
pub trait Failure {
    fn span(&self) -> Option<&SpanType>;
    fn msg(&self) -> String;
    fn info(&self) -> &Vec<Info>;
    fn ty(&self) -> FailureType;
    fn is_critical(&self) -> bool {
        match self.ty() {
            FailureType::ParseError
            | FailureType::CheckerError
            | FailureType::TokenizerError
            | FailureType::RuntimeError => true,
            FailureType::Warning => false,
        }
    }
}

impl Failure for RuntimeError {
    fn span(&self) -> Option<&SpanType> {
        Some(&self.span)
    }

    fn msg(&self) -> String {
        format!("{}: {}", self.ty, self.msg.clone())
    }

    fn info(&self) -> &Vec<Info> {
        &self.info
    }

    fn ty(&self) -> FailureType {
        FailureType::RuntimeError
    }
}

impl Failure for Diagnostic {
    fn span(&self) -> Option<&SpanType> {
        self.span.as_ref()
    }

    fn msg(&self) -> String {
        format!("{}", self.ty)
    }

    fn info(&self) -> &Vec<Info> {
        &self.info
    }

    fn ty(&self) -> FailureType {
        match self.ty {
            DiagType::Err(_, stage) => match stage {
                Stage::Tokenizer => FailureType::TokenizerError,
                Stage::Parser => FailureType::ParseError,
                Stage::Checker => FailureType::CheckerError,
            },
            DiagType::Warn(_) => FailureType::Warning,
        }
    }
}
impl Display for SpanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpanType::AutoGenerated => write!(f, "<auto generated>"),
            SpanType::Syntax(span) => write!(f, "{span}"),
        }
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
        if let Some(SpanType::Syntax(v)) = &self.span {
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
    pub span: Option<SpanType>,
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
