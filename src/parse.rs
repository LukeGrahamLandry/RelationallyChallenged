//! Convert a stream of tokens to an AstExpr (or multiple)

use crate::ir::{AstExpr, AstValue};
use crate::lex::{LexErr, Lexer, Token, TokenType};

pub struct Parser<'src> {
    lex: Lexer<'src>,
}

#[derive(Debug, Clone)]
pub enum ParseErr<'a> {
    WantFound(TokenType<'a>, Token<'a>),
    WantVal(Token<'a>),
    Lex(LexErr),
}

type Res<'a, T> = Result<T, ParseErr<'a>>;

impl<'src> Parser<'src> {
    pub fn new(lex: Lexer<'src>) -> Self {
        Self { lex }
    }

    pub fn next(&mut self) -> Res<AstExpr<'src>> {
        let t = self.lex.next()?;
        match t.0 {
            TokenType::Ident(name) => {
                self.require(TokenType::Equals)?;
                self.require(TokenType::LeftSquiggle)?;
                let headers = self.parse_headers()?;
                Ok(AstExpr::CreateNamedTable {
                    data: self.parse_rows(headers.len())?,
                    name,
                    headers,
                })
            }
            _ => Err(ParseErr::WantFound(TokenType::Ident(""), t)),
        }
    }

    fn parse_headers(&mut self) -> Res<'src, Vec<&'src str>> {
        let mut names = vec![];
        loop {
            names.push(self.require_ident()?);
            if self.lex.nth(0)?.0 == TokenType::Comma {
                self.lex.next()?;
            } else {
                break;
            }
        }
        Ok(names)
    }

    fn parse_rows(&mut self, width: usize) -> Res<'src, Vec<Vec<AstValue<'src>>>> {
        let mut rows = vec![];
        while self.lex.nth(0)?.0 != TokenType::RightSquiggle {
            let mut row = vec![];
            row.push(self.parse_value()?);
            for _ in 0..(width - 1) {
                // TODO: better error if row wrong length. maybe fn for list of comma seperated tokens and reuse for header
                self.require(TokenType::Comma)?;
                row.push(self.parse_value()?);
            }
            rows.push(row)
        }
        self.require(TokenType::RightSquiggle)?;
        Ok(rows)
    }

    fn parse_value(&mut self) -> Res<'src, AstValue<'src>> {
        let t = self.lex.next()?;
        match t.0 {
            TokenType::Ident(s) | TokenType::Quoted(s) => Ok(AstValue::Str(s)),
            TokenType::Number(n) => Ok(AstValue::Num(n)),
            _ => Err(ParseErr::WantVal(t)),
        }
    }

    // TODO: awkward can only do no payload
    fn require(&mut self, t: TokenType<'src>) -> Res<'src, ()> {
        let found = self.lex.next()?;
        if found.0 != t {
            return Err(ParseErr::WantFound(t, found));
        }
        Ok(())
    }

    fn require_ident(&mut self) -> Res<'src, &'src str> {
        let found = self.lex.next()?;
        if let TokenType::Ident(v) = found.0 {
            return Ok(v);
        }
        return Err(ParseErr::WantFound(TokenType::Ident(""), found));
    }
}

// TODO: ugh can i derive dumb wrappers?
impl<'a> From<LexErr> for ParseErr<'a> {
    fn from(value: LexErr) -> Self {
        ParseErr::Lex(value)
    }
}
