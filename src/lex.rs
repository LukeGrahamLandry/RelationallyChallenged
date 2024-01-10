use crate::lex::TokenType::*;
use std::iter::Peekable;
use std::num::ParseFloatError;
use std::ops::Range;
use std::str::Chars;

// TODO: why tf is Range not copy
#[derive(Debug, Clone)]
pub struct Token<'a>(pub TokenType<'a>, pub Range<usize>);

#[derive(Debug, Copy, Clone)]
pub enum TokenType<'a> {
    Ident(&'a str),
    Number(f64),
    Quoted(&'a str),
    Equals,
    LeftSquiggle,
    RightSquiggle,
    LeftParen,
    RightParen,
    Pi,
    Comma,
}

pub type Res<T> = Result<T, LexErr>;

#[derive(Debug, Clone)]
pub enum LexErr {
    EOF,             // not always an error
    UnterminatedStr, // need pos
    Unexpected(char),
    Num(ParseFloatError), // TODO: need substr
}

pub struct Lexer<'a> {
    src: &'a str,
    start: usize,
    current: usize,
    chars: Peekable<Chars<'a>>,
    peeked: Option<Res<Token<'a>>>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            start: 0,
            current: 0,
            chars: src.chars().peekable(),
            peeked: None,
        }
    }

    pub fn next(&mut self) -> Res<Token<'a>> {
        if let Some(prev) = self.peeked.take() {
            return prev;
        }

        self.eat_whitespace()?;
        self.start = self.current;
        match self.peek_c()? {
            '"' | '“' | '”' | '\'' => self.lex_quoted(),
            '0'..='9' => self.lex_num(),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_ident(),
            '{' => self.one(LeftSquiggle),
            '}' => self.one(RightSquiggle),
            '(' => self.one(LeftParen),
            ')' => self.one(RightParen),
            '=' => self.one(Equals),
            ',' => self.one(Comma),
            'Π' | 'π' => self.one(Pi),
            c => Err(LexErr::Unexpected(c)),
        }
    }

    pub fn peek(&mut self) -> Res<Token<'a>> {
        match self.peeked.clone() {
            None => {
                let next = self.next();
                self.peeked = Some(next.clone());
                next
            }
            Some(prev) => prev,
        }
    }

    fn one(&mut self, t: TokenType<'static>) -> Res<Token<'a>> {
        self.pop()?;
        Ok(Token(t, self.start..self.current))
    }

    // TODO: disallow multiline?
    // TODO: support escape characters
    fn lex_quoted(&mut self) -> Res<Token<'a>> {
        self.pop()?;
        loop {
            match self.pop() {
                Ok('"' | '“' | '”' | '\'') => {
                    // Payload doesn't include quotes.
                    let text = &self.src[self.start + 1..self.current - 1];
                    return Ok(Token(Quoted(text), self.start..self.current));
                }
                Err(LexErr::EOF) => return Err(LexErr::UnterminatedStr),
                Ok(_) => {}
                _ => unreachable!(),
            }
        }
    }

    fn lex_num(&mut self) -> Res<Token<'a>> {
        loop {
            let done = match self.peek_c() {
                Ok(c) => !c.is_numeric() && c != '.',
                Err(LexErr::EOF) => true,
                Err(_) => unreachable!(),
            };
            if done {
                let text = &self.src[self.start..self.current];
                let n = match text.parse::<f64>() {
                    Ok(n) => n,
                    Err(err) => return Err(LexErr::Num(err)), // TODO: ugly! use Into
                };
                return Ok(Token(Number(n), self.start..self.current));
            }
            self.pop()?;
        }
    }

    fn lex_ident(&mut self) -> Res<Token<'a>> {
        let token = self.lex_ident_raw()?;
        let ty = match token.0 {
            "pi" => Pi,
            text => Ident(text),
        };
        Ok(Token(ty, token.1))
    }

    fn lex_ident_raw(&mut self) -> Res<(&'a str, Range<usize>)> {
        loop {
            let c = self.peek_c()?;
            let done = !c.is_ascii_alphanumeric() && !matches!(c, '_');
            if done {
                let text = &self.src[self.start..self.current];
                return Ok((text, self.start..self.current));
            }
            self.pop()?;
        }
    }

    fn eat_whitespace(&mut self) -> Res<()> {
        while self.peek_c()?.is_whitespace() {
            self.pop()?;
        }
        Ok(())
    }

    fn peek_c(&mut self) -> Res<char> {
        self.chars.peek().copied().ok_or(LexErr::EOF)
    }

    fn pop(&mut self) -> Res<char> {
        match self.chars.next() {
            None => Err(LexErr::EOF),
            Some(c) => {
                self.current += 1;
                Ok(c)
            }
        }
    }
}
