use std::{
    str::CharIndices, 
    iter::Peekable
};

pub (crate) struct Tokenizer<'i> {
    source: &'i str,
    chars: Peekable<CharIndices<'i>>,
    current_position: Position,
}

impl<'i> Tokenizer<'i> {
    pub (crate) fn new(source: &'i str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            current_position: Position::default()
        }
    }

    fn match_number(&mut self, start_pos: Position) -> Result<LoxToken, LoxParseErr> {
        self.match_char_while(|(_, ch)| ch.is_ascii_digit());
        if self.match_char('.') {
            if !self.match_char_if(|(_, ch)| ch.is_ascii_digit()) {
                return Err(LoxParseErr { kind: ErrKind::TrailingPeriod, span: Span { start: self.current_position, end: self.peek_position() } })
            }
            self.match_char_while(|(_, ch)| ch.is_ascii_digit());
        }

        return Ok(LoxToken { kind: TokenKind::Number, span: Span { start: start_pos, end: self.peek_position() } })
    }

    /// ASSUMES that the beginning quote was already consumed.
    fn match_string(&mut self) -> Result<LoxToken, LoxParseErr> {
        let mut previous_was_backslash = false;
        let start_pos = self.current_position;
        self.match_char_while(|(_, ch)| {
            match ch {
                '\\' => {
                    previous_was_backslash = true;
                    true
                }
                '"' => {
                    if !previous_was_backslash {
                        // break the loop
                        false
                    } else {
                        previous_was_backslash = false;
                        true    
                    }
                }
                _ => {
                    previous_was_backslash = false;
                    true
                }
            }
        });

        if !self.match_char('"') {
            return Err(LoxParseErr { kind: ErrKind::UnexpectedEOF, span: Span { start: start_pos, end: self.peek_position() } });
        }

        return Ok(LoxToken { kind: TokenKind::String, span: Span { start: start_pos, end: self.peek_position() } });
    }

    fn match_identifier(&mut self, start_pos: Position) -> LoxToken {
        self.match_char_if(|(_, ch)| {
            match ch {
                'a'..='z'
                | 'A'..='Z'
                | '_' => true,
                _ => false,
            }
        });
        
        self.match_char_while(|(_, ch)| {
            match ch {
                'a'..='z'
                | 'A'..='Z'
                | '0'..='9'
                | '_' => true,
                _ => false,
            }
        });

        let ident = &self.source[self.current_position.byte..self.peek_position().byte];
        let kind = match ident {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "fun" => TokenKind::Fun,
            "for" => TokenKind::For,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        };

        LoxToken {
            kind,
            span: Span {
                start: start_pos,
                end: self.peek_position()
            }
        }
    }

    fn match_char_while<P: FnMut(&(usize, char)) -> bool>(&mut self, mut predicate: P) {
        loop {
            if !self.match_char_if(&mut predicate) { break; }
        }
    }

    fn match_char(&mut self, ch: char) -> bool {
        self.match_char_if(|ch_index| ch_index.1 == ch)
    }

    fn match_char_if<P: FnMut(&(usize, char)) -> bool>(&mut self, mut predicate: P) -> bool {
        match self.chars.peek() {
            None => false,
            Some(ch_index) => {
                if predicate(ch_index) {
                    // consume it
                    self.next_char();
                    true
                } else {
                    false
                }
            },
        }
    }

    /// Gets the next character, updating our position tracking and bookkeeping in the process.
    fn next_char(&mut self) -> Option<(usize, char)> {
        match self.chars.next() {
            None => {
                self.current_position = Position {
                    // don't increment the line or column, since reporting a position that is off the page is confusing to users.
                    line: self.current_position.line,
                    col: self.current_position.col,
                    byte: self.source.as_bytes().len(),
                };
                None
            }
            Some(ch_index) => {
                match ch_index.1 {
                    '\n' => {
                        self.current_position = Position {
                            line: self.current_position.line + 1,
                            col: 0,
                            byte: ch_index.0
                        };
                        Some(ch_index)
                    }
                    _ => {
                        self.current_position = Position {
                            line: self.current_position.line,
                            col: self.current_position.col + 1,
                            byte: ch_index.0
                        };
                        Some(ch_index)
                    }
                }                
            }
        }
    }

    fn peek_position(&mut self) -> Position {
        match self.chars.peek() {
            None => Position {
                line: self.current_position.line,
                // don't increment the line or column, since reporting a position that is off the page is confusing to users.
                col: self.current_position.col, 
                byte: self.source.as_bytes().len(),
            },
            Some(ch_index) => {
                match ch_index.1 {
                    '\n' => {
                        Position {
                            line: self.current_position.line + 1,
                            col: 0,
                            byte: ch_index.0
                        }
                    }
                    _ => {
                        Position {
                            line: self.current_position.line,
                            col: self.current_position.col + 1,
                            byte: ch_index.0
                        }
                    }
                }                
            }
        }
    }
}

impl<'i> Iterator for Tokenizer<'i> {
    type Item = Result<LoxToken, LoxParseErr>;

    /// Fetches the next token. None signifies EOF.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.next_char() {
                None => return None,
                Some(ch_index) => {
                    let token_start = self.current_position;
                    match ch_index.1 {
                        '(' =>  return Some(Ok(LoxToken { kind: TokenKind::LeftParen,   span: Span { start: token_start, end: self.peek_position() } })),
                        ')' =>  return Some(Ok(LoxToken { kind: TokenKind::RightParen,  span: Span { start: token_start, end: self.peek_position() } })),
                        '{' =>  return Some(Ok(LoxToken { kind: TokenKind::LeftBrace,   span: Span { start: token_start, end: self.peek_position() } })),
                        '}' =>  return Some(Ok(LoxToken { kind: TokenKind::RightBrace,  span: Span { start: token_start, end: self.peek_position() } })),
                        ',' =>  return Some(Ok(LoxToken { kind: TokenKind::Comma,       span: Span { start: token_start, end: self.peek_position() } })),
                        '.' =>  return Some(Ok(LoxToken { kind: TokenKind::Dot,         span: Span { start: token_start, end: self.peek_position() } })),
                        '-' =>  return Some(Ok(LoxToken { kind: TokenKind::Minus,       span: Span { start: token_start, end: self.peek_position() } })),
                        '+' =>  return Some(Ok(LoxToken { kind: TokenKind::Plus,        span: Span { start: token_start, end: self.peek_position() } })),
                        ';' =>  return Some(Ok(LoxToken { kind: TokenKind::Semicolon,   span: Span { start: token_start, end: self.peek_position() } })),
                        '/' =>  return Some(Ok(LoxToken { kind: TokenKind::Slash,       span: Span { start: token_start, end: self.peek_position() } })),
                        '*' =>  return Some(Ok(LoxToken { kind: TokenKind::Star,        span: Span { start: token_start, end: self.peek_position() } })),
                        '!' => {
                            if self.match_char('=') {
                                return Some(Ok(LoxToken { kind: TokenKind::BangEqual,   span: Span { start: token_start, end: self.peek_position() } }))
                            } else {
                                return Some(Ok(LoxToken { kind: TokenKind::Bang,        span: Span { start: token_start, end: self.peek_position() } }))
                            }
                        }
                        '>' => {
                            if self.match_char('=') {
                                return Some(Ok(LoxToken { kind: TokenKind::GreaterEqual,   span: Span { start: token_start, end: self.peek_position() } }))
                            } else {
                                return Some(Ok(LoxToken { kind: TokenKind::Greater,        span: Span { start: token_start, end: self.peek_position() } }))
                            }
                        }
                        '<' => {
                            if self.match_char('=') {
                                return Some(Ok(LoxToken { kind: TokenKind::LessEqual,   span: Span { start: token_start, end: self.peek_position() } }))
                            } else {
                                return Some(Ok(LoxToken { kind: TokenKind::Less,        span: Span { start: token_start, end: self.peek_position() } }))
                            }
                        }
                        '0'..='9' => {
                            return Some(self.match_number(token_start))
                        }
                        '"' => {
                            return Some(self.match_string())
                        }
                        'a'..='z'
                        | 'A'..='Z'
                        | '_' => {
                            return Some(Ok(self.match_identifier(token_start)))
                        }
                        other => {
                            if other.is_ascii_whitespace() { continue; }
                            // TODO: panic mode recovery.
                            return Some(Err(LoxParseErr { kind: ErrKind::InvalidChar, span: Span { start: token_start, end: self.peek_position() } }))
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LoxToken {
    kind: TokenKind,
    span: Span,
}

#[derive(Clone, Copy, Debug, Default)]
struct Span {
    start: Position,
    end: Position,
}

#[derive(Clone, Copy, Debug)]
enum TokenKind {
    // Single char unambiguous
    LeftParen, // (
    RightParen, // )
    LeftBrace, // {
    RightBrace, // }
    Comma, // ,
    Dot, // .
    Minus, // -
    Plus, // +
    Semicolon, // ;
    Slash, // /
    Star, // *
    
    // Single or double char ambiguous
    Bang, // !
    BangEqual, // !=
    Greater, // >
    GreaterEqual, // >=
    Less, // <
    LessEqual, // <=

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While
}

#[derive(Clone, Copy, Debug, Default)]
struct Position {
    line: usize,
    col: usize,
    byte: usize
}

#[derive(Clone, Copy, Debug)]
pub (crate) struct LoxParseErr {
    kind: ErrKind,
    span: Span
}

#[derive(Clone, Copy, Debug)]
enum ErrKind {
    InvalidChar,
    /// This error signifies a number that ended in a period. This is invalid syntax.
    TrailingPeriod,
    UnexpectedEOF,
}