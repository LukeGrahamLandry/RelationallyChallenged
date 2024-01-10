use std::iter::Peekable;
use std::num::ParseFloatError;
use std::ops::Range;
use std::str::Chars;

// TODO: why tf is Range not copy
#[derive(Debug, Clone)]
pub struct Token<'a>(TokenType<'a>, Range<usize>);

#[derive(Debug, Copy, Clone)]
pub enum TokenType<'a> {
    Ident(&'a str),
    Number(f64),
    String(&'a str),
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
    Num(ParseFloatError),
}

pub struct Lexer<'a> {
    src: &'a str,
    start: usize,
    current: usize,
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            start: 0,
            current: 0,
            chars: src.chars().peekable(),
        }
    }

    pub fn next(&mut self) -> Res<Token> {
        self.eat_whitespace()?;
        self.start = self.current;
        match self.peek()? {
            '"' | '“' | '”' | '\'' => self.lex_quoted(),
            '0'..='9' => self.lex_num(),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_ident(),
            '{' => self.one(TokenType::LeftSquiggle),
            '}' => self.one(TokenType::RightSquiggle),
            '(' => self.one(TokenType::LeftParen),
            ')' => self.one(TokenType::RightParen),
            '=' => self.one(TokenType::Equals),
            ',' => self.one(TokenType::Comma),
            'Π' | 'π' => self.one(TokenType::Pi),
            c => Err(LexErr::Unexpected(c)),
        }
    }

    fn one(&mut self, t: TokenType<'static>) -> Res<Token> {
        self.pop()?;
        Ok(Token(t, self.start..self.current))
    }

    // TODO: disallow multiline?
    // TODO: support escape characters
    fn lex_quoted(&mut self) -> Res<Token> {
        self.pop()?;
        loop {
            match self.pop()? {
                '"' | '“' | '”' | '\'' => {
                    // Payload doesn't include quotes.
                    let text = &self.src[self.start + 1..self.current - 1];
                    return Ok(Token(TokenType::String(text), self.start..self.current));
                }
                _ => {}
            }
        }
    }

    fn lex_num(&mut self) -> Res<Token> {
        loop {
            let c = self.pop()?;
            let done = !c.is_numeric() && c != '.';
            if done {
                let text = &self.src[self.start..self.current - 1];
                let n = match text.parse::<f64>() {
                    Ok(n) => n,
                    Err(err) => return Err(LexErr::Num(err)), // TODO: ugly! use Into
                };
                return Ok(Token(TokenType::Number(n), self.start..self.current));
            }
        }
    }

    fn lex_ident(&mut self) -> Res<Token> {
        let token = self.lex_ident_raw()?;
        let ty = match token.0 {
            "pi" => TokenType::Pi,
            text => TokenType::Ident(text),
        };
        Ok(Token(ty, token.1))
    }

    fn lex_ident_raw(&mut self) -> Res<(&str, Range<usize>)> {
        loop {
            let c = self.pop()?;
            let done = !c.is_ascii_alphanumeric() && !matches!(c, '_');
            if done {
                let text = &self.src[self.start..self.current - 1];
                return Ok((text, self.start..self.current));
            }
        }
    }

    fn eat_whitespace(&mut self) -> Res<()> {
        while self.peek()?.is_whitespace() {
            self.pop()?;
        }
        Ok(())
    }

    fn peek(&mut self) -> Res<char> {
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
