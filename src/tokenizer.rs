use std::{cell::LazyCell, collections::HashMap, fmt::Display, rc::Rc};

use heck::{ToPascalCase, ToUpperCamelCase};
use ordered_float::NotNan;
use strum::EnumDiscriminants;

use crate::types::{DiagType, Diagnostic, ErrType, Info, Span, WarnType};

#[derive(Debug, Clone, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(name(TokenKind))]
pub enum TokenType {
    StringLiteral(String),
    Int(i32),
    Newline,
    Ident(String),
    Equals,
    Plus,
    Division,
    Multiplication,
    NotEqualsTo,
    Minus,
    Comma,
    ArrowRL,
    Input,
    Output,
    Declare,
    If,
    IntegerType,
    RealType,
    CharType,
    BooleanType,
    StringType,
    Colon,
    Else,
    Then,
    Endif,
    LParen,
    RParen,
    Modulo,
    IntegerDivide,
    LT,
    GT,
    LTE,
    GTE,
    LogicalOr,
    LogicalAnd,
    LogicalNot,
    CaseOf,
    Real(NotNan<f64>),
    False,
    True,
    Otherwise,
    Endcase,
}

impl TokenType {
    pub fn kind(&self) -> TokenKind {
        self.into()
    }
}
impl TokenKind {
    pub fn str(&self) -> Option<&str> {
        Some(match self {
            TokenKind::Input => "INPUT",
            TokenKind::Output => "OUTPUT",
            TokenKind::Declare => "DECLARE",
            TokenKind::If => "IF",
            TokenKind::IntegerType => "INTEGER",
            TokenKind::RealType => "REAL",
            TokenKind::CharType => "CHAR",
            TokenKind::BooleanType => "BOOLEAN",
            TokenKind::StringType => "STRING",
            TokenKind::Else => "ELSE",
            TokenKind::Then => "THEN",
            TokenKind::Endif => "ENDIF",
            TokenKind::Modulo => "MOD",
            TokenKind::IntegerDivide => "DIV",
            TokenKind::LogicalOr => "OR",
            TokenKind::LogicalAnd => "AND",
            TokenKind::LogicalNot => "NOT",
            TokenKind::CaseOf => "CASE OF",
            TokenKind::False => "FALSE",
            TokenKind::True => "TRUE",
            TokenKind::Otherwise => "OTHERWISE",
            TokenKind::Endcase => "ENDCASE",
            _ => None?,
        })
    }
}
thread_local! {
    static MAP_SINGLE: LazyCell<HashMap<char, TokenType>> = LazyCell::new(|| {
        let mut m = HashMap::new();
        m.insert('=', TokenType::Equals);
        m.insert('>', TokenType::GT);
                m.insert('<', TokenType::LT);

        m.insert('+', TokenType::Plus);
        m.insert('-', TokenType::Minus);
        m.insert('/', TokenType::Division);
        m.insert('*', TokenType::Multiplication);
        m.insert(',', TokenType::Comma);
        m.insert(':', TokenType::Colon);
        m.insert('(', TokenType::LParen);
        m.insert(')', TokenType::RParen);
        m
    });
}

thread_local! {
    static MAP_DOUBLE: LazyCell<HashMap<(char, char), TokenType>> = LazyCell::new(|| {
        let mut m = HashMap::new();
        m.insert(('<', '-'), TokenType::ArrowRL);
        m.insert(('I', 'F'), TokenType::If);
        m.insert(('>', '='), TokenType::GTE);
        m.insert(('<', '='), TokenType::LTE);
        m.insert(('O', 'R'), TokenType::LogicalOr);
        m.insert(('!', '='), TokenType::NotEqualsTo);

        m
    });
}

#[derive(Debug, Clone)]
pub struct Token {
    pub span: Span,
    pub token: TokenType,
}

impl TokenType {
    pub fn str_value(&self) -> &String {
        match self {
            TokenType::StringLiteral(s) => s,
            TokenType::Ident(i) => i,
            _ => unreachable!("as_ident called with a different token than Identifier or String"),
        }
    }
}

impl Token {
    pub fn str_value(&self) -> &String {
        self.token.str_value()
    }
}
impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&TokenType as Into<TokenKind>>::into(self))
    }
}
impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::StringLiteral => write!(f, "string"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Newline => write!(f, "newline"),
            TokenKind::Ident => write!(f, "identifier"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Division => write!(f, "/"),
            TokenKind::Multiplication => write!(f, "*"),
            TokenKind::NotEqualsTo => write!(f, "<>"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Comma => write!(f, "comma"),
            TokenKind::ArrowRL => write!(f, "<-"),
            TokenKind::Input => write!(f, "INPUT"),
            TokenKind::Output => write!(f, "OUTPUT"),
            TokenKind::Declare => write!(f, "DECLARE"),
            TokenKind::If => write!(f, "IF"),
            TokenKind::IntegerType => write!(f, "INTEGER"),
            TokenKind::RealType => write!(f, "REAL"),
            TokenKind::CharType => write!(f, "CHAR"),
            TokenKind::BooleanType => write!(f, "BOOLEAN"),
            TokenKind::StringType => write!(f, "STRING"),
            TokenKind::Colon => write!(f, "colon (:)"),
            TokenKind::Else => write!(f, "ELSE"),
            TokenKind::Then => write!(f, "THEN"),
            TokenKind::Endif => write!(f, "ENDIF"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Modulo => write!(f, "MOD"),
            TokenKind::LT => write!(f, "<"),
            TokenKind::GT => write!(f, ">"),
            TokenKind::LTE => write!(f, "<="),
            TokenKind::GTE => write!(f, ">="),
            TokenKind::IntegerDivide => write!(f, "//"),
            TokenKind::LogicalOr => write!(f, "OR"),
            TokenKind::LogicalAnd => write!(f, "AND"),
            TokenKind::LogicalNot => write!(f, "NOT"),
            TokenKind::CaseOf => write!(f, "CASE OF"),
            TokenKind::Otherwise => write!(f, "OTHERWISE"),
            TokenKind::Real => write!(f, "REAL"),
            TokenKind::False => write!(f, "TRUE"),
            TokenKind::True => write!(f, "FALSE"),
            TokenKind::Endcase => write!(f, "ENDCASE"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Tokenizer {
    current_span: Span,
    current_char: Option<char>,
    previous_span: Option<Span>,
    content: String,
    diag_queue: Vec<Diagnostic>,
}
fn is_all_caps(string: &str) -> bool {
    for char in string.chars() {
        if !char.is_uppercase() {
            return false;
        }
    }
    true
}
impl Iterator for Tokenizer {
    type Item = Result<Option<Token>, Diagnostic>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(diag) = self.diag_queue.pop() {
            return Some(Err(diag));
        }
        while let Some(c) = self.current_char
            && c.is_whitespace()
            && c != '\n'
        {
            self.advance();
        }
        if self.current_char.is_none() {
            return None;
        }
        Some(self.next_token())
    }
}

impl Tokenizer {
    fn skip_line(&mut self) {
        while let Some(t) = self.current_char
            && t != '\n'
        {
            self.advance();
        }
        self.advance();
    }
    pub fn next_token(&mut self) -> Result<Option<Token>, Diagnostic> {
        let c = self.expect_char()?;
        let start = self.current_span.clone();
        if c == '"' {
            return self.tokenize_string().map(|i| Some(i));
        } else if c.is_numeric() {
            return self.tokenize_numeric().map(|n| Some(n));
        } else if c == '/' && self.peek(1) == Some('/') {
            let span = self.current_span.clone();
            self.skip_line();
            return Ok(Some(Token {
                span,
                token: TokenType::Newline,
            }));
        } else if c == '\n' {
            self.advance();
            return Ok(Some(Token {
                token: TokenType::Newline,
                span: self.prev_span(),
            }));
        } else if MAP_DOUBLE.with(|m| self.peek(1).map_or(false, |v| m.contains_key(&(c, v)))) {
            let token = MAP_DOUBLE.with(|m| m[&(c, self.peek(1).unwrap())].clone());
            self.advance_multiple(2);
            return Ok(Some(Token {
                token,
                span: start.merge(self.prev_span()),
            }));
        } else if MAP_SINGLE.with(|m| m.contains_key(&c)) {
            let token = MAP_SINGLE.with(|m| m[&c].clone());
            self.advance();
            return Ok(Some(Token {
                token,
                span: self.prev_span(),
            }));
        } else if self.check("INPUT") {
            // TODO: remove all this duplication
            self.advance_multiple(5);
            return Ok(Some(Token {
                token: TokenType::Input,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("DECLARE") {
            self.advance_multiple(7);
            return Ok(Some(Token {
                token: TokenType::Declare,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("INTEGER") {
            self.advance_multiple(7);
            return Ok(Some(Token {
                token: TokenType::IntegerType,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("REAL") {
            self.advance_multiple(4);
            return Ok(Some(Token {
                token: TokenType::RealType,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("BOOOLEAN") {
            self.advance_multiple(8);
            return Ok(Some(Token {
                token: TokenType::BooleanType,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("CHAR") {
            self.advance_multiple(4);
            return Ok(Some(Token {
                token: TokenType::CharType,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("STRING") {
            self.advance_multiple(6);
            return Ok(Some(Token {
                token: TokenType::StringType,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("OUTPUT") {
            self.advance_multiple(6);
            return Ok(Some(Token {
                token: TokenType::Output,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("ELSE") {
            self.advance_multiple(4);
            return Ok(Some(Token {
                token: TokenType::Else,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("THEN") {
            self.advance_multiple(4);
            return Ok(Some(Token {
                token: TokenType::Then,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("ENDIF") {
            self.advance_multiple(6);
            return Ok(Some(Token {
                token: TokenType::Endif,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("AND") {
            self.advance_multiple(3);
            return Ok(Some(Token {
                token: TokenType::LogicalAnd,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("NOT") {
            self.advance_multiple(6);
            return Ok(Some(Token {
                token: TokenType::LogicalNot,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("MOD") {
            self.advance_multiple(3);
            return Ok(Some(Token {
                token: TokenType::Modulo,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("CASE OF") {
            self.advance_multiple(7);
            return Ok(Some(Token {
                token: TokenType::CaseOf,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("TRUE") {
            self.advance_multiple(4);
            return Ok(Some(Token {
                token: TokenType::True,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("FALSE") {
            self.advance_multiple(5);
            return Ok(Some(Token {
                token: TokenType::False,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("OTHERWISE") {
            self.advance_multiple(9);
            return Ok(Some(Token {
                token: TokenType::Otherwise,
                span: start.merge(self.prev_span()),
            }));
        } else if self.check("ENDCASE") {
            self.advance_multiple(7);
            return Ok(Some(Token {
                token: TokenType::Endcase,
                span: start.merge(self.prev_span()),
            }));
        } else if c.is_alphabetic() {
            return self.tokenize_ident().map(|i| Some(i));
        }
        return Err(Diagnostic {
            ty: DiagType::Err(ErrType::InvalidChar {
                char: self.current_char,
            }),
            info: vec![Info::note(
                "this character is never used in cambridge pseudocode",
            )],
            span: Some(self.current_span.clone()),
        });
    }

    fn tokenize_numeric(&mut self) -> Result<Token, Diagnostic> {
        let start = self.current_span.clone();
        let mut out = String::new();
        let mut real = false;
        while let Some(t) = self.current_char
            && t.is_numeric()
        {
            out.push(t);
            self.advance();
        }
        if let Some(t) = self.current_char
            && t == '.'
        {
            real = true;
            self.advance();
            out.push('.');
            while let Some(t) = self.current_char
                && t.is_numeric()
            {
                out.push(t);
                self.advance();
            }
        }
        if real {
            let parsed = out.parse().unwrap(); // as the chars were checked against `is_numeric()`, this is safe
            Ok(Token {
                span: start.merge(self.prev_span()),
                token: TokenType::Real(parsed),
            })
        } else {
            let parsed = out.parse().unwrap();
            Ok(Token {
                span: start.merge(self.prev_span()),
                token: TokenType::Int(parsed),
            })
        }
    }
    fn tokenize_ident(&mut self) -> Result<Token, Diagnostic> {
        let mut out = String::new();
        let start = self.current_span.clone();
        while let Some(c) = self.current_char
            && (c.is_alphanumeric() || c == '_')
        {
            out.push(c);
            self.advance();
        }
        
        if out != out.to_pascal_case() {
            self.diag_queue.push(Diagnostic {
                ty: DiagType::Warn(WarnType::UnreccomendedVariableName),
                info: vec![Info::note("variable names should use UpperCamelCase"), Info::help(format!("a reccomended name would be {}", out.to_upper_camel_case()))],
                span: Some(start.merge(self.prev_span())),
            })
        }
        return Ok(Token {
            span: start.merge(self.prev_span()),
            token: TokenType::Ident(out),
        });
    }
    fn tokenize_string(&mut self) -> Result<Token, Diagnostic> {
        let start = self.current_span.clone();
        self.advance();
        let mut out = String::new();
        let mut broken = false;
        while self.expect_char()? != '\n' {
            let c = self.current_char.unwrap();
            if c == '"' {
                broken = true;
                self.advance();
                break;
            }
            out.push(c);
            self.advance();
        }
        if !broken {
            return Err(Diagnostic {
                ty: DiagType::Err(ErrType::UnterminatedStringLiteral),
                info: vec![Info::note("you never ended the string")],
                span: Some(start.merge(self.prev_span())),
            });
        }
        return Ok(Token {
            span: start.merge(self.prev_span()),
            token: TokenType::StringLiteral(out),
        });
    }
    fn prev_span(&self) -> Span {
        match &self.previous_span {
            Some(s) => s.clone(),
            None => Span {
                ln: 0,
                col: 0,
                pos: 0,
                endln: None,
                endcol: None,
                endpos: None,
                fp: self.current_span.fp.clone(),
            },
        }
    }
    fn peek(&self, lookahead: usize) -> Option<char> {
        self.content.chars().nth(self.pos() + lookahead)
    }
    fn check(&self, check: &'static str) -> bool {
        for (i, c) in check.chars().enumerate() {
            if !self.peek(i).is_some_and(|f| f == c) {
                return false;
            }
        }
        true
    }
}
impl Tokenizer {
    fn eof(&self) -> Diagnostic {
        Diagnostic {
            ty: DiagType::Err(ErrType::UnexpectedEOF),
            info: vec![Info::note("EOF stands for End Of File")],
            span: Some(self.current_span.clone()),
        }
    }
    fn expect_char(&self) -> Result<char, Diagnostic> {
        if let Some(c) = self.current_char {
            Ok(c)
        } else {
            Err(self.eof())
        }
    }
    pub fn new(content: String, fp: impl Into<Rc<str>>) -> Self {
        Self {
            current_char: content.chars().nth(0),
            content,
            current_span: Span {
                ln: 0,
                col: 0,
                pos: 0,
                endln: None,
                endcol: None,
                endpos: None,
                fp: fp.into(),
            },
            previous_span: None,
            diag_queue: Vec::new(),
        }
    }
    pub fn advance(&mut self) -> Option<char> {
        self.advance_multiple(1)
    }
    pub fn advance_multiple(&mut self, amnt: usize) -> Option<char> {
        let mut advanced = None;
        for _ in 0..amnt {
            self.previous_span = Some(self.current_span.clone());
            if let Some(c) = self.content.chars().nth(self.pos())
                && c == '\n'
            {
                *self.ln_mut() += 1;
                *self.col_mut() = 0;
            } else {
                *self.col_mut() += 1;
            }
            *self.pos_mut() += 1;
            advanced = self.content.chars().nth(self.pos());
        }
        self.current_char = self.content.chars().nth(self.pos());
        advanced
    }
}

impl Tokenizer {
    fn pos(&self) -> usize {
        self.current_span.pos
    }
    fn pos_mut(&mut self) -> &mut usize {
        &mut self.current_span.pos
    }
    // fn ln(&self) -> usize {
    //     self.current_span.ln
    // }
    fn ln_mut(&mut self) -> &mut usize {
        &mut self.current_span.ln
    }
    // fn col(&self) -> usize {
    //     self.current_span.col
    // }
    fn col_mut(&mut self) -> &mut usize {
        &mut self.current_span.col
    }
}
