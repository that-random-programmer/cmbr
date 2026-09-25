use std::{fmt::Display, rc::Rc};

use crate::{
    tokenizer::{Token, TokenKind, TokenType},
    treewalker::Type,
    types::{DiagType, Diagnostic, ErrType, Info, Span},
};

#[derive(Debug, Clone)]
pub struct Parser {
    tkns: Vec<Token>,
    current: Option<Token>,
    previous: Option<Token>,
    span: Span,
    previous_span: Span,
    pos: usize,
    fp: Rc<str>,
}

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub ty: ASTNodeType,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub enum ASTNodeType {
    If {
        condition: Expr,
        if_block: Block,
        else_block: Option<Block>,
    },
    Input {
        ident: (String, Span),
        prompt: Option<Expr>,
    },
    Expr(Expr),
    Output(Vec<Expr>),
    Declare {
        ident: String,
        ty: Type,
    },
    Case {
        value: Expr,
        cases: Vec<((Atom, Span), Vec<ASTNode>)>,
        otherwise: Option<Vec<ASTNode>>,
    },
    Assign {
        ident: (String, Span),
        value: Expr
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOpType {
    LogicalNot,
    Negate,
    Positive,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOpType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    LogicalOr,
    LogicalAnd,
    Equality,
    InEquality,
    LT,
    LTE,
    GT,
    GTE,
    IntegerDivide,
}
impl TryFrom<&TokenType> for BinOpType {
    type Error = ();

    fn try_from(value: &TokenType) -> Result<Self, Self::Error> {
        type B = BinOpType;
        Ok(match value {
            TokenType::Plus => B::Add,
            TokenType::Minus => B::Subtract,
            TokenType::Multiplication => B::Multiply,
            TokenType::Division => B::Divide,
            TokenType::Modulo => B::Modulo,
            TokenType::LogicalOr => B::LogicalOr,
            TokenType::LogicalAnd => B::LogicalAnd,
            TokenType::Equals => B::Equality,
            TokenType::NotEqualsTo => B::InEquality,
            TokenType::LT => B::LT,
            TokenType::GT => B::GT,
            TokenType::LTE => B::LTE,
            TokenType::GTE => B::GTE,
            TokenType::IntegerDivide => B::IntegerDivide,
            _ => return Err(()),
        })
    }
}

impl TryFrom<&TokenType> for UnaryOpType {
    type Error = Diagnostic;

    fn try_from(value: &TokenType) -> Result<Self, Self::Error> {
        type U = UnaryOpType;
        Ok(match value {
            TokenType::Plus => U::Positive,
            TokenType::Minus => U::Negate,
            TokenType::LogicalNot => U::LogicalNot,
            _ => panic!("internal compiler error: invalid UnaryOp from {value}"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Block(pub Vec<ASTNode>);

impl Iterator for Parser {
    type Item = Result<ASTNode, Diagnostic>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Atom {
    String(String),
    Int(i32),
    Ident(String),
    Real(f64),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub ty: ExprType,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub enum ExprType {
    Atom(Atom),
    BinOp(BinOpType, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOpType, Box<Expr>),
}
impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::String(s) => write!(f, "\"{s}\""),
            Atom::Int(i) => write!(f, "{i}"),
            Atom::Ident(n) => write!(f, "{n}"),
            Atom::Real(r) => write!(f, "{r}"),
            Atom::Boolean(b) => write!(f, "{}", b.to_string().to_uppercase()),
        }
    }
}
impl Display for BinOpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOpType::Add => write!(f, "+"),
            BinOpType::Subtract => write!(f, "-"),
            BinOpType::Multiply => write!(f, "*"),
            BinOpType::Divide => write!(f, "/"),
            BinOpType::Modulo => write!(f, "MOD"),
            BinOpType::LogicalOr => write!(f, "OR"),
            BinOpType::LogicalAnd => write!(f, "AND"),
            BinOpType::Equality => write!(f, "="),
            BinOpType::InEquality => write!(f, "!="),
            BinOpType::LT => write!(f, "<"),
            BinOpType::LTE => write!(f, "<="),
            BinOpType::GT => write!(f, ">"),
            BinOpType::GTE => write!(f, ">="),
            BinOpType::IntegerDivide => write!(f, "//"),
        }
    }
}

impl Display for UnaryOpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOpType::LogicalNot => write!(f, "NOT"),
            UnaryOpType::Negate => write!(f, "-"),
            UnaryOpType::Positive => write!(f, "+"),
        }
    }
}
impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}
impl Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprType::Atom(satom) => write!(f, "{satom}"),
            ExprType::BinOp(ty, item1, item2) => {
                write!(f, "({ty} {item1} {item2})")
            }
            ExprType::UnaryOp(ty, item) => {
                write!(f, "({ty} {item})")
            }
        }
    }
}

fn prefix_binding_power(op: UnaryOpType) -> ((), u8) {
    match op {
        UnaryOpType::Negate | UnaryOpType::Positive | UnaryOpType::LogicalNot => ((), 13),
    }
}
fn infix_binding_power(op: BinOpType) -> (u8, u8) {
    match op {
        BinOpType::Add | BinOpType::Subtract => (9, 10),
        BinOpType::Multiply | BinOpType::Divide | BinOpType::Modulo | BinOpType::IntegerDivide => {
            (11, 12)
        }
        BinOpType::LogicalOr => (1, 2),
        BinOpType::LogicalAnd => (3, 4),
        BinOpType::Equality | BinOpType::InEquality => (5, 6),
        BinOpType::LT | BinOpType::GT | BinOpType::LTE | BinOpType::GTE => (7, 8),
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::Tokenizer;

    use super::*;
    fn parse(expr: impl Into<String>) -> Result<Expr, Diagnostic> {
        let content = expr.into();
        let tk = Tokenizer::new(content, Rc::from("<TODO>"));
        let mut tkns = Vec::new();
        for tkn in tk {
            if let Some(v) = tkn.unwrap() {
                tkns.push(v)
            }
        }
        Parser::new(tkns, "<TODO>").expr()
    }
    #[test]
    fn expr() {
        assert_eq!(
            parse("5 + 2 * 6 / 6").unwrap().to_string().as_str(),
            "(+ 5 (/ (* 2 6) 6))"
        )
    }
}

const STATIC_ATOM: [TokenKind; 5] = [
    TokenKind::Int,
    TokenKind::Real,
    TokenKind::StringLiteral,
    TokenKind::False,
    TokenKind::True,
];
const ATOM: [TokenKind; 6] = [
    TokenKind::Int,
    TokenKind::Real,
    TokenKind::StringLiteral,
    TokenKind::False,
    TokenKind::True,
    TokenKind::Ident,
];
impl Parser {
    pub fn parse(&mut self) -> Result<Vec<ASTNode>, Diagnostic> {
        let mut out = Vec::new();
        while !self.current.is_none() {
            self.skip_newline();
            out.push(self.stmnt()?)
        }
        Ok(out)
    }
    fn skip_newline(&mut self) {
        while let Some(t) = &self.current
            && t.token == TokenType::Newline
        {
            self.advance();
        }
    }
    fn parse_if(&mut self, inner: bool) -> Result<ASTNode, Diagnostic> {
        let start = self.span.clone();
        self.advance();
        let condition = self.expr()?;
        self.eat(&TokenKind::Then)?;
        let mut stmts = Vec::new();
        self.skip_newline();
        while !matches!(
            self.expect_token()?.token,
            TokenType::Endif | TokenType::Else
        ) {
            stmts.push(self.stmnt()?);
        }
        let if_block = Block(stmts);
        let mut else_block = None;
        self.skip_newline();
        while self.expect_token()?.token == TokenType::Else {
            self.advance();
            self.skip_newline();
            let mut else_stmnt = Vec::new();
            if self.expect_token()?.token == TokenType::If {
                else_stmnt.push(self.parse_if(true)?);
            } else {
                while !matches!(
                    self.expect_token()?.token,
                    TokenType::Endif | TokenType::Else
                ) {
                    else_stmnt.push(self.stmnt()?);
                }
            }

            else_block = Some(Block(else_stmnt))
        }
        if !inner {
            self.eat(&TokenKind::Endif)?;
        }
        Ok(ASTNode {
            ty: ASTNodeType::If {
                condition,
                if_block,
                else_block,
            },
            span: start.merge(self.previous_span.clone()),
        })
    }
    fn can_continue_case(&mut self) -> bool {
        let mut i = 0;
        while let Some(t) = self.peek_amnt(i)
            && t.token == TokenType::Newline
        {
            i += 1;
        }
        match self.peek_amnt(i) {
            None => return false,
            Some(v) => {
                if STATIC_ATOM.contains(&v.token.kind()) {
                    i += 1;
                    match self.peek_amnt(i) {
                        None => return false,
                        Some(v) => return v.token == TokenType::Colon,
                    }
                } else {
                    return false;
                }
            }
        }
    }
    pub fn stmnt(&mut self) -> Result<ASTNode, Diagnostic> {
        self.skip_newline();
        let start = self.span.clone();
        match self.expect_token()?.token {
            TokenType::Input => {
                self.advance();
                let ident = (
                    self.eat(&TokenKind::Ident)?.str_value().to_owned(),
                    self.previous_span.clone(),
                );
                let mut prompt = None;
                if self.eat(&TokenKind::Comma).is_ok() {
                    prompt = Some(self.expr()?)
                }
                let s = start.merge(self.previous_span.clone());
                self.newline(s.clone())?;
                return Ok(ASTNode {
                    ty: ASTNodeType::Input { ident, prompt },
                    span: start.merge(s.clone()),
                });
            }
            TokenType::Ident(ident_value) if self.peek_amnt(1).is_some_and(|t| t.token == TokenType::ArrowRL) => {
                let ident = (ident_value, self.span.clone());
                self.advance();
                self.advance();
                let value = self.expr()?;
                Ok(ASTNode {
                    ty: ASTNodeType::Assign {
                        ident,
                        value
                    },
                    span: start.merge(self.previous_span.clone())
                })
            }
            TokenType::Output => {
                self.advance();
                let mut out = vec![self.expr()?];
                while let Some(t) = &self.current
                    && matches!(t.token, TokenType::Comma)
                {
                    self.advance();
                    out.push(self.expr()?)
                }
                let s = start.merge(self.previous_span.clone());
                self.newline(s.clone())?;
                return Ok(ASTNode {
                    ty: ASTNodeType::Output(out),
                    span: s,
                });
            }
            TokenType::If => self.parse_if(false),
            TokenType::CaseOf => {
                self.advance();
                let value = self.expr()?;
                self.skip_newline();
                let mut cases = Vec::new();
                let mut otherwise = None;
                while !matches!(
                    self.expect_token()?.token,
                    TokenType::Endcase | TokenType::Otherwise
                ) {
                    let condition = self.static_atom()?;
                    let span = self.span.clone();
                    self.advance();
                    let mut stmnts = Vec::new();
                    self.eat(&TokenKind::Colon)?;
                    while !self.can_continue_case()
                        && !matches!(
                            self.expect_token()?.token,
                            TokenType::Otherwise | TokenType::Endcase
                        )
                    {
                        stmnts.push(self.stmnt()?);
                    }
                    self.skip_newline();
                    cases.push(((condition, span), stmnts))
                }
                if self.expect_token()?.token == TokenType::Otherwise {
                    self.advance();
                    let mut stmnts = Vec::new();
                    while self.expect_token()?.token != TokenType::Endcase {
                        stmnts.push(self.stmnt()?)
                    }
                    otherwise = Some(stmnts)
                }
                self.eat(&TokenKind::Endcase)?;
                Ok(ASTNode {
                    ty: ASTNodeType::Case {
                        value,
                        cases,
                        otherwise,
                    },
                    span: start.merge(self.previous_span.clone()),
                })
            }
            TokenType::Declare => {
                self.advance();
                let ident = self.eat(&TokenKind::Ident)?.str_value().to_owned();
                self.eat(&TokenKind::Colon)?;
                let ty = match self.expect_token()?.token {
                    TokenType::StringType => Type::String,
                    TokenType::IntegerType => Type::Int,
                    TokenType::BooleanType => Type::Bool,
                    TokenType::CharType => Type::Char,
                    TokenType::RealType => Type::Real,
                    tkn => {
                        return Err(Diagnostic {
                            ty: DiagType::Err(ErrType::UnexpectedToken {
                                tkn,
                                expected: Some(vec![
                                    TokenKind::StringType,
                                    TokenKind::IntegerType,
                                    TokenKind::BooleanType,
                                    TokenKind::CharType,
                                    TokenKind::RealType,
                                ]),
                            }),
                            info: vec![Info::note("expected a type")],
                            span: Some(self.span.clone()),
                        });
                    }
                };
                self.advance();
                self.newline(start.merge(self.previous_span.clone()))?;
                return Ok(ASTNode {
                    ty: ASTNodeType::Declare { ident, ty },
                    span: start.merge(self.previous_span.clone()),
                });
            }
            _ => Ok(ASTNode {
                ty: ASTNodeType::Expr(self.expr()?),
                span: start.merge(self.previous_span.clone()),
            }),
        }
    }
    fn static_atom(&mut self) -> Result<Atom, Diagnostic> {
        match self.expect_token()?.token {
            TokenType::Int(v) => Ok(Atom::Int(v)),
            TokenType::Real(v) => Ok(Atom::Real(*v)),
            TokenType::StringLiteral(v) => Ok(Atom::String(v.clone())),
            TokenType::False => Ok(Atom::Boolean(false)),
            TokenType::True => Ok(Atom::Boolean(true)),
            t => Err(Diagnostic {
                ty: DiagType::Err(ErrType::UnexpectedToken {
                    tkn: t,
                    expected: Some(STATIC_ATOM.to_vec()),
                }),
                info: vec![],
                span: Some(self.span.clone()),
            }),
        }
    }
    pub fn expr(&mut self) -> Result<Expr, Diagnostic> {
        self.expr_r(0)
    }

    pub fn expr_r(&mut self, min_p: u8) -> Result<Expr, Diagnostic> {
        let mut lhs;
        let start = self.span.clone();
        match self.static_atom() {
            Ok(v) => {
                self.advance();
                lhs = Expr {
                    ty: ExprType::Atom(v),
                    span: self.previous_span.clone(),
                }
            }
            Err(_) => {
                lhs = match self.expect_token()?.token {
                    TokenType::Ident(a) => {
                        self.advance();
                        Expr {
                            ty: ExprType::Atom(Atom::Ident(a)),
                            span: self.previous_span.clone(),
                        }
                    }
                    TokenType::LParen => {
                        self.advance();
                        let lhs = self.expr_r(0)?;
                        self.eat(&TokenKind::RParen)?;
                        lhs
                    }
                    op @ (TokenType::Plus | TokenType::Minus) => {
                        self.advance();

                        let unary = UnaryOpType::try_from(&op)?;
                        let ((), rp) = prefix_binding_power(unary);
                        let rhs = self.expr_r(rp)?;
                        Expr {
                            ty: ExprType::UnaryOp(unary, Box::new(rhs)),
                            span: start.merge(self.previous_span.clone()),
                        }
                    }
                    tkn => {
                        let mut expected = ATOM.to_vec();
                        expected.push(TokenKind::LParen);

                        return Err(Diagnostic {
                            ty: DiagType::Err(ErrType::UnexpectedToken {
                                tkn,
                                expected: Some(expected),
                            }),
                            info: vec![],
                            span: Some(self.span.clone()),
                        });
                    }
                }
            }
        };
        loop {
            let op = match &self.current {
                None => break,
                Some(t) => match BinOpType::try_from(&t.token) {
                    Err(_) => break,
                    Ok(v) => v,
                },
            };
            let (lp, rp) = infix_binding_power(op);
            if lp < min_p {
                break;
            }
            self.advance();
            let rhs = self.expr_r(rp)?;
            lhs = Expr {
                ty: ExprType::BinOp(op, Box::new(lhs), Box::new(rhs)),
                span: start.merge(self.previous_span.clone()),
            };
        }
        Ok(lhs)
    }
    // fn peek(&self) -> Option<&Token> {
    //     self.peek_amnt(1)
    // }
    fn peek_amnt(&self, amnt: usize) -> Option<&Token> {
        self.tkns.get(self.pos + amnt)
    }
    // fn expect_peek(&self) -> Result<&Token, Diagnostic> {
    //     match self.peek() {
    //         Some(v) => Ok(v),
    //         None => Err(Diagnostic {
    //             ty: DiagType::Err(ErrType::UnexpectedEOF),
    //             info: vec![Info::note("EOF stands for End Of File")],
    //             span: Some(self.span.clone()),
    //         }),
    //     }
    // }
    fn newline(&mut self, span: Span) -> Result<(), Diagnostic> {
        match &self.current {
            None => return Ok(()),
            Some(v) => match &v.token {
                TokenType::Newline => {
                    self.advance();
                    return Ok(());
                }
                _ => {
                    return Err(Diagnostic {
                        ty: DiagType::Err(ErrType::ExpectedNewline),
                        info: vec![Info::help("consider adding a newline after this statement")],
                        span: Some(span),
                    });
                }
            },
        }
    }
    fn advance(&mut self) -> Option<Token> {
        self.pos += 1;
        self.previous = self.current.clone();
        self.previous_span = self.span.clone();
        self.current = self.tkns.get(self.pos).cloned();
        self.span = self
            .current
            .as_ref()
            .map_or_else(|| Span::empty(self.fp.clone()), |t| t.span.clone());
        self.current.clone()
    }
    fn eat(&mut self, ty: &TokenKind) -> Result<Token, Diagnostic> {
        self.eat_info(ty, &[])
    }
    fn eat_info(&mut self, ty: &TokenKind, info: &[Info]) -> Result<Token, Diagnostic> {
        let current = self.expect_token()?;
        if current.token.kind() == *ty {
            self.advance();
            return Ok(current);
        } else {
            return Err(Diagnostic {
                ty: DiagType::Err(ErrType::UnexpectedToken {
                    tkn: current.token,
                    expected: Some(vec![*ty]),
                }),
                info: info.to_vec(),
                span: Some(self.span.clone()),
            });
        }
    }
    pub fn expect_token(&self) -> Result<Token, Diagnostic> {
        match &self.current {
            Some(v) => Ok(v.clone()),
            None => Err(Diagnostic {
                ty: DiagType::Err(ErrType::UnexpectedEOF),
                info: vec![Info::note("EOF stands for End Of File")],
                span: Some(self.span.clone()),
            }),
        }
    }
    pub fn new(tkns: impl Into<Vec<Token>>, fp: impl Into<Rc<str>>) -> Self {
        let tkns = tkns.into();
        let fp = fp.into();
        Self {
            current: tkns.get(0).cloned(),
            fp: fp.clone(),
            span: tkns
                .get(0)
                .map(|t| t.span.clone())
                .unwrap_or_else(|| Span::empty(fp.clone())),
            tkns: tkns,
            previous: None,
            pos: 0,
            previous_span: Span::empty(fp),
        }
    }
}
