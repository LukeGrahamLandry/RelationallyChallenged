//! Convert a stream of tokens to an AstExpr (or multiple)

use crate::ir::{AstExpr, AstPred, AstValue, BinOp, Loc};
use crate::lex::{LexErr, Lexer, Token, TokenType};
use std::ops::Range;

pub struct Parser<'src> {
    lex: Lexer<'src>,
    locs: Vec<Range<usize>>,
}

#[derive(Debug, Clone)]
pub enum ParseErr<'a> {
    WantFound(TokenType<'a>, Token<'a>),
    WantVal(Token<'a>),
    WantExpr(Token<'a>),
    Lex(LexErr),
}

type Res<'a, T> = Result<T, ParseErr<'a>>;

impl<'src> Parser<'src> {
    pub fn new(lex: Lexer<'src>) -> Self {
        Self { lex, locs: vec![] }
    }

    pub fn next(&mut self) -> Res<'src, Loc<AstExpr<'src>>> {
        let t = self.lex.nth(0)?;
        match t.0 {
            TokenType::Ident(name) => {
                self.push_loc()?;
                self.lex.next()?;
                self.require(TokenType::Equals)?;
                self.require(TokenType::LeftSquiggle)?;
                let headers = self.ident_list()?;
                let data = self.parse_rows(headers.len())?;
                let loc = self.pop_loc()?;
                Ok((
                    AstExpr::CreateNamedTable {
                        data,
                        name,
                        headers,
                    },
                    loc,
                ))
            }
            _ => self.parse_table_expr(),
        }
    }

    fn parse_table_expr(&mut self) -> Res<'src, Loc<AstExpr<'src>>> {
        let t = self.lex.next()?;
        match t.0 {
            TokenType::Ident(name) => Ok((AstExpr::GetNamedTable(name), t.1)),
            TokenType::Pi => {
                let fields = self.ident_list()?;
                let input = Box::new(self.parse_table_expr()?);
                let end = input.1.start;
                Ok((AstExpr::TakeFields { input, fields }, t.1.start..end))
            }
            TokenType::Sigma => {
                let cond = self.pred()?;
                let input = Box::new(self.parse_table_expr()?);
                let loc = t.1.start..input.1.end;
                Ok((AstExpr::Filter { input, cond }, loc))
            }
            TokenType::LeftParen => {
                let expr = self.parse_table_expr()?;
                self.require(TokenType::RightParen)?;
                Ok(expr)
            }
            _ => todo!("err"),
        }
    }

    fn pred(&mut self) -> Res<'src, AstPred<'src>> {
        let left = self.pred_primary()?;
        match self.lex.nth(0)?.0 {
            TokenType::Equals => {
                self.require(TokenType::Equals)?;
                let right = self.pred()?;
                Ok(AstPred::Bin(BinOp::Eq, Box::new(left), Box::new(right)))
            }
            _ => Ok(left),
        }
    }

    fn pred_primary(&mut self) -> Res<'src, AstPred<'src>> {
        let t = self.lex.next()?;
        match t.0 {
            TokenType::Ident(name) => Ok(AstPred::Field(name)),
            TokenType::Number(n) => Ok(AstPred::Literal(AstValue::Num(n))),
            TokenType::Quoted(s) => Ok(AstPred::Literal(AstValue::Str(s))),
            TokenType::LeftParen => {
                let e = self.pred()?;
                self.require(TokenType::RightParen)?;
                Ok(e)
            }
            _ => Err(ParseErr::WantExpr(t)),
        }
    }

    fn comma_sep(&mut self) -> Res<'src, Vec<Token<'src>>> {
        let mut tokens = vec![];
        loop {
            tokens.push(self.lex.next()?); // TODO: only some things are valid. want to catch trailing comma
            if self.lex.nth(0)?.0 == TokenType::Comma {
                self.lex.next()?;
            } else {
                break;
            }
        }
        Ok(tokens)
    }

    fn ident_list(&mut self) -> Res<'src, Vec<&'src str>> {
        Ok(self
            .comma_sep()?
            .into_iter()
            .map(|t| match t.0 {
                TokenType::Ident(s) => s,
                _ => todo!("err, header wanted ident"),
            })
            .collect())
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

    fn push_loc(&mut self) -> Res<'src, ()> {
        self.locs.push(self.lex.nth(0)?.1);
        Ok(())
    }

    fn pop_loc(&mut self) -> Res<'src, Range<usize>> {
        let start = self.locs.pop().unwrap().start; // TODO: err
        let end = self.lex.nth(0)?.1.start; // TODO: want last token not next
        Ok(start..end)
    }
}

// TODO: ugh can i derive dumb wrappers?
impl<'a> From<LexErr> for ParseErr<'a> {
    fn from(value: LexErr) -> Self {
        ParseErr::Lex(value)
    }
}
