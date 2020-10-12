use std::collections::VecDeque;

use kg_diag::*;
use kg_diag::parse::*;

pub type Error = ParseDiag;

pub type Token = LexToken<Terminal>;

#[derive(Debug, Display, Detail)]
#[diag(code_offset = 600)]
pub enum ParseErrorDetail {

}

impl ParseErrorDetail {

}

#[inline]
pub(crate) fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[inline]
pub(crate) fn is_non_ident_char(c: Option<char>) -> bool {
    match c {
        None => true,
        Some(c) => !is_ident_char(c),
    }
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Copy)]
pub enum Terminal {
    #[display(fmt = "END")]
    End,
    #[display(fmt = "','")]
    Comma,
    #[display(fmt = "';'")]
    Semicolon,
    #[display(fmt = "'('")]
    ParenLeft,
    #[display(fmt = "')'")]
    ParenRight,
    #[display(fmt = "'['")]
    BracketLeft,
    #[display(fmt = "']'")]
    BracketRight,
    #[display(fmt = "'{{'")]
    BraceLeft,
    #[display(fmt = "'}}'")]
    BraceRight,
    #[display(fmt = "'mod'")]
    KwMod,
    #[display(fmt = "'fn'")]
    KwFn,
    #[display(fmt = "'let'")]
    KwLet,
    #[display(fmt = "'if'")]
    KwIf,
    #[display(fmt = "'else'")]
    KwElse,
    #[display(fmt = "'for'")]
    KwFor,
    #[display(fmt = "ID")]
    Id,
    #[display(fmt = "string literal")]
    String,
    #[display(fmt = "integer literal")]
    IntDecimal,
    #[display(fmt = "hex integer literal")]
    IntHex,
    #[display(fmt = "octal integer literal")]
    IntOctal,
    #[display(fmt = "binary integer literal")]
    IntBinary,
    #[display(fmt = "float literal")]
    Float,
    #[display(fmt = "'true'")]
    True,
    #[display(fmt = "'false'")]
    False,
    #[display(fmt = "'null'")]
    Null,
}

impl LexTerm for Terminal {}

#[derive(Debug)]
pub struct Parser {
    num_parser: NumberParser,
    prev_pos: Position,
    next_pos: Position,
    token_queue: VecDeque<Token>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            num_parser: {
                let mut p = NumberParser::new();
                p.decimal.allow_plus = false;
                p.decimal.allow_minus = false;
                p.hex.allow_plus = false;
                p.hex.allow_minus = false;
                p.octal.allow_plus = false;
                p.octal.allow_minus = false;
                p.binary.allow_plus = false;
                p.binary.allow_minus = false;
                p
            },
            prev_pos: Position::default(),
            next_pos: Position::default(),
            token_queue: VecDeque::new(),
        }
    }

    fn lex(&mut self, r: &mut dyn CharReader) -> Result<Token, Error> {
        fn consume(r: &mut dyn CharReader, count: usize, term: Terminal) -> Result<Token, Error> {
            let p1 = r.position();
            r.skip_chars(count)?;
            let p2 = r.position();
            Ok(Token::new(term, p1, p2))
        }

        r.skip_whitespace()?;

        if self.num_parser.is_at_start(r)? {
            let n = self.num_parser.parse_number(r)?;
            match n.term().notation() {
                Notation::Decimal => Ok(Token::new(Terminal::IntDecimal, n.from(), n.to())),
                Notation::Hex => Ok(Token::new(Terminal::IntHex, n.from(), n.to())),
                Notation::Octal => Ok(Token::new(Terminal::IntOctal, n.from(), n.to())),
                Notation::Binary => Ok(Token::new(Terminal::IntBinary, n.from(), n.to())),
                Notation::Float | Notation::Exponent => Ok(Token::new(Terminal::Float, n.from(), n.to())),
            }
        } else {
            match r.peek_char(0)? {
                None => Ok(Token::new(Terminal::End, r.position(), r.position())),
                Some(',') => consume(r, 1, Terminal::Comma),
                Some('(') => consume(r, 1, Terminal::ParenLeft),
                Some(')') => consume(r, 1, Terminal::ParenRight),
                Some('[') => consume(r, 1, Terminal::BracketLeft),
                Some(']') => consume(r, 1, Terminal::BracketRight),
                Some('{') => consume(r, 1, Terminal::BraceLeft),
                Some('}') => consume(r, 1, Terminal::BraceRight),
                Some(';') => consume(r, 1, Terminal::Semicolon),
                Some('n') => {
                    if r.match_str_term("null", &mut is_non_ident_char)? {
                        let p1 = r.position();
                        r.skip_chars(4)?;
                        let p2 = r.position();
                        Ok(Token::new(Terminal::Null, p1, p2))
                    } else {
                        let p1 = r.position();
                        r.next_char()?;
                        r.skip_while(&mut is_ident_char)?;
                        let p2 = r.position();
                        Ok(Token::new(Terminal::Id, p1, p2))
                    }
                }
                Some('t') => {
                    if r.match_str_term("true", &mut is_non_ident_char)? {
                        let p1 = r.position();
                        r.skip_chars(4)?;
                        let p2 = r.position();
                        Ok(Token::new(Terminal::True, p1, p2))
                    } else {
                        let p1 = r.position();
                        r.next_char()?;
                        r.skip_while(&mut is_ident_char)?;
                        let p2 = r.position();
                        Ok(Token::new(Terminal::Id, p1, p2))
                    }
                }
                Some('f') => {
                    if r.match_str_term("false", &mut is_non_ident_char)? {
                        let p1 = r.position();
                        r.skip_chars(5)?;
                        let p2 = r.position();
                        Ok(Token::new(Terminal::False, p1, p2))
                    } else {
                        let p1 = r.position();
                        r.next_char()?;
                        r.skip_while(&mut is_ident_char)?;
                        let p2 = r.position();
                        Ok(Token::new(Terminal::Id, p1, p2))
                    }
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    let p1 = r.position();
                    r.next_char()?;
                    r.skip_while(&mut is_ident_char)?;
                    let p2 = r.position();
                    Ok(Token::new(Terminal::Id, p1, p2))
                }
                Some(c) if c == '\'' || c == '\"' => {
                    let p1 = r.position();
                    while let Some(k) = r.next_char()? {
                        if k == '\\' {
                            r.next_char()?;
                        } else if k == c {
                            break;
                        }
                    }
                    if r.eof() {
                        ParseErrorDetail::invalid_input_one(r, c)
                    } else {
                        r.next_char()?;
                        let p2 = r.position();
                        Ok(Token::new(Terminal::String, p1, p2))
                    }
                }
                Some(_c) => {
                    let p1 = r.position();

                    // (jc) do not report lex errors when parsing partial input
                    if self.partial {
                        Ok(Token::new(Terminal::End, p1, p1))
                    } else {
                        ParseErrorDetail::invalid_input(r)
                    }
                }
            }
        }
    }

    fn next_token(&mut self, r: &mut dyn CharReader) -> Result<Token, Error> {
        if self.token_queue.is_empty() {
            let t = self.lex(r)?;
            self.prev_pos = self.next_pos;
            self.next_pos = t.to();
            Ok(t)
        } else {
            let t = self.token_queue.pop_front().unwrap();
            self.next_pos = t.to();
            Ok(t)
        }
    }

    fn push_token(&mut self, t: Token) {
        self.next_pos = self.prev_pos;
        self.token_queue.push_back(t);
    }

    fn expect_token(&mut self, r: &mut dyn CharReader, term: Terminal) -> Result<Token, Error> {
        let t = self.next_token(r)?;
        if t.term() == term {
            Ok(t)
        } else {
            ParseErrorDetail::unexpected_token_one(t, term, r)
        }
    }

    fn expect_token_many(&mut self, r: &mut dyn CharReader, terms: &[Terminal]) -> Result<Token, Error> {
        let t = self.next_token(r)?;
        if terms.contains(&t.term()) {
            Ok(t)
        } else {
            ParseErrorDetail::unexpected_token_many(t, terms.to_vec(), r)
        }
    }

    pub fn parse(&mut self, r: &mut dyn CharReader) -> Result<Opath, Error> {
        let p = r.position();
        self.token_queue.clear();
        self.next_pos = p;

        let e = self.parse_expr(r, Context::Expr);

        match e {
            Ok(e) => {
                if self.partial {
                    r.seek(self.next_pos)?;
                }
                Ok(Opath::new(e))
            }
            Err(err) => {
                if self.partial {
                    r.seek(p)?;
                }
                Err(err)
            }
        }
    }
}