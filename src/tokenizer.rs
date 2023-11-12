use std::{
    str::CharIndices, 
    iter::Peekable, fmt::Display, u8, ops::Range, error::Error
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
        let start = self.current_position;
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

        let ident = &self.source[start.byte..self.peek_position().byte];
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
                        '/' =>  {
                            // single line comment
                            if self.match_char('/') {
                                self.match_char_while(|ch| ch.1 != '\n');
                                self.match_char('\n');
                                // We don't return comment tokens.
                            // multi line comment
                            } else if self.match_char('*') {
                                loop {
                                    self.match_char_while(|ch| ch.1 != '*');
                                    if self.match_char('*') && self.match_char('/') {
                                        break;
                                    }
                                    // Break if we hit EOF.
                                    if self.chars.peek().is_none() { break; }
                                }
                                // We don't return comment tokens.
                            // otherwise, division
                            } else {
                                return Some(Ok(LoxToken { kind: TokenKind::Slash,       span: Span { start: token_start, end: self.peek_position() } }));
                            }
                        }
                        '*' =>  return Some(Ok(LoxToken { kind: TokenKind::Star,        span: Span { start: token_start, end: self.peek_position() } })),
                        '!' => {
                            if self.match_char('=') {
                                return Some(Ok(LoxToken { kind: TokenKind::BangEqual,   span: Span { start: token_start, end: self.peek_position() } }))
                            } else {
                                return Some(Ok(LoxToken { kind: TokenKind::Bang,        span: Span { start: token_start, end: self.peek_position() } }))
                            }
                        }
                        '=' => {
                            if self.match_char('=') {
                                return Some(Ok(LoxToken { kind: TokenKind::EqualEqual,   span: Span { start: token_start, end: self.peek_position() } }))
                            } else {
                                return Some(Ok(LoxToken { kind: TokenKind::Equal,        span: Span { start: token_start, end: self.peek_position() } }))
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

impl LoxToken {
    pub (crate) fn get_start(&self) -> Position {
        self.span.start
    }

    pub (crate) fn kind(&self) -> TokenKind {
        self.kind
    }

    pub (crate) fn range(&self) -> Range<usize> {
        self.span.start.byte..self.span.end.byte
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Span {
    start: Position,
    end: Position,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub (crate) enum TokenKind {
    LeftParen = 0, // (
    RightParen = 1, // )
    LeftBrace = 2, // {
    RightBrace = 3, // }
    Comma = 4, // ,
    Dot = 5, // .
    Minus = 6, // -
    Plus = 7, // +
    Semicolon = 8, // ;
    Slash = 9, // /
    Star = 10, // *
    Bang = 11, // !
    BangEqual = 12, // !=
    Equal = 36, // =
    EqualEqual = 37, // ==
    Greater = 13, // >
    GreaterEqual = 14, // >=
    Less = 15, // <
    LessEqual = 16, // <=
    Identifier = 17,
    String = 18,
    Number = 19,
    And = 20,
    Class = 21,
    Else = 22,
    False = 23,
    Fun = 24,
    For = 25,
    If = 26,
    Nil = 27,
    Or = 28,
    Print = 29,
    Return = 30,
    Super = 31,
    This = 32,
    True = 33,
    Var = 34,
    While = 35,
    /// This value is never yielded by the tokenizer, but is useful for error reporting.
    EOF = 38,
}

impl TokenKind {
    pub const fn max() -> usize {
        TokenKind::While as usize
    }
}

impl TryFrom<u8> for TokenKind {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TokenKind::LeftParen),
            1 => Ok(TokenKind::RightParen),
            2 => Ok(TokenKind::LeftBrace),
            3 => Ok(TokenKind::RightBrace),
            4 => Ok(TokenKind::Comma),
            5 => Ok(TokenKind::Dot),
            6 => Ok(TokenKind::Minus),
            7 => Ok(TokenKind::Plus),
            8 => Ok(TokenKind::Semicolon),
            9 => Ok(TokenKind::Slash),
            10 => Ok(TokenKind::Star),
            11 => Ok(TokenKind::Bang),
            12 => Ok(TokenKind::BangEqual),
            13 => Ok(TokenKind::Greater),
            14 => Ok(TokenKind::GreaterEqual),
            15 => Ok(TokenKind::Less),
            16 => Ok(TokenKind::LessEqual),
            17 => Ok(TokenKind::Identifier),
            18 => Ok(TokenKind::String),
            19 => Ok(TokenKind::Number),
            20 => Ok(TokenKind::And),
            21 => Ok(TokenKind::Class),
            22 => Ok(TokenKind::Else),
            23 => Ok(TokenKind::False),
            24 => Ok(TokenKind::Fun),
            25 => Ok(TokenKind::For),
            26 => Ok(TokenKind::If),
            27 => Ok(TokenKind::Nil),
            28 => Ok(TokenKind::Or),
            29 => Ok(TokenKind::Print),
            30 => Ok(TokenKind::Return),
            31 => Ok(TokenKind::Super),
            32 => Ok(TokenKind::This),
            33 => Ok(TokenKind::True),
            34 => Ok(TokenKind::Var),
            35 => Ok(TokenKind::While),
            other => Err(other)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub (crate) struct Position {
    line: usize,
    col: usize,
    byte: usize
}

impl Position {
    pub (crate) fn line(&self) -> usize {
        self.line
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, column: {}", self.line, self.col)
    }
}

#[derive(Clone, Copy, Debug)]
pub (crate) struct LoxParseErr {
    kind: ErrKind,
    span: Span
}

impl LoxParseErr {
    pub (crate) fn get_start(&self) -> Position {
        self.span.start
    }
}

impl Display for LoxParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line: {}, column: {}] {:?}", self.span.start.line, self.span.start.col, self.kind)
    }
}
impl Error for LoxParseErr {}

#[derive(Clone, Copy, Debug)]
enum ErrKind {
    InvalidChar,
    /// This error signifies a number that ended in a period. This is invalid syntax.
    TrailingPeriod,
    UnexpectedEOF,
}