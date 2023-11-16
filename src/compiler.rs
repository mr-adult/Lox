use std::{error::Error, fmt::{Debug, Display}, vec::IntoIter, iter::Peekable, collections::HashSet, rc::Rc};

use tree_iterators_rs::prelude::{BinaryTreeNode, OwnedBinaryTreeNode};

use crate::{
    tokenizer::{
        Tokenizer, 
        LoxToken, LoxParseErr, Position, TokenKind
    }, 
    chunk::{Chunk, OpCode}, value::Value, object::Object, fixed_vec::FixedVec, vm::STACK_MAX
};

pub (crate) fn compile(source: &str) -> Result<(Vec<Chunk>, FixedVec<Value, STACK_MAX>), Vec<CompileErr>> {
    let token_stream = Tokenizer::new(source);
    let mut errs = None;
    let mut tokens = Vec::new();
    for token_result in token_stream {
        match token_result {
            Err(err) => {
                match &mut errs {
                    None => errs = Some(Vec::new()),
                    Some(errs) => errs.push(err),
                }
            }
            Ok(token) => tokens.push(token),
        }
    }
    
    if let Some(errs) = errs {
        // Found an error. Don't go any further.
        return Err(errs.into_iter().map(|err| {
            CompileErr {
                kind: CompileErrKind::Parse(err),
                location: err.get_start()
            }
        }).collect());
    }

    let compiler = Compiler::new(FunctionType::Script, None, source, tokens);
    compiler.compile()
}

struct Compiler<'c> {
    enclosing: Option<&'c Self>,
    f_type: FunctionType,
    state: CompilerState,

    chunks: Vec<Chunk>,
    errs: Vec<CompileErr>,
    values: FixedVec<Value, STACK_MAX>,

    source_code: &'c str,
    tokens: Option<Peekable<IntoIter<LoxToken>>>,

    previous: Option<LoxToken>,
    current: Option<LoxToken>,
}

impl<'c> Compiler<'c> {
    fn new(f_type: FunctionType, enclosing: Option<&'c Self>, source_code: &'c str, tokens: Vec<LoxToken>) -> Self {
        Self {
            enclosing: enclosing, 
            f_type,
            previous: None,
            current: None,
            tokens: Some(tokens.into_iter().peekable()),
            source_code: source_code,
            chunks: Vec::new(),
            errs: Vec::new(),
            state: CompilerState::Declaration,
            values: FixedVec::<_, STACK_MAX>::new(),
        }
    }

    fn compile(mut self) -> Result<(Vec<Chunk>, FixedVec<Value, STACK_MAX>), Vec<CompileErr>> {
        while let Some(_) = self.tokens
                .as_mut()
                .expect("tokenizer to be yielded to active Compiler")
                .peek()
        {
            match self.declaration() {
                Ok(()) => {},
                Err(()) => self.panic_mode_recovery(),
            }
        }

        self.chunks.push(self.chunk(OpCode::Return as u8));
    
        if self.errs.len() > 0 {
            Err(self.errs)
        } else {
            Ok((self.chunks, self.values))
        }
    }

    fn declaration(&mut self) -> Result<(), ()> {
        while self.tokens
            .as_mut()
            .expect("tokens to be in active compiler")
            .peek()
            .is_some() {
            
            self.statement()?;
        }

        Ok(())
    }

    fn statement(&mut self) -> Result<(), ()> {
        if self.match_token(TokenKind::Class) {
            todo!();
        } else if self.match_token(TokenKind::Fun) {
            todo!();
        } else if self.match_token(TokenKind::Print) {
            self.expression_statement()?;
            self.chunks.push(self.chunk(OpCode::Print as u8));
            if !self.match_token(TokenKind::Semicolon) {
                self.errs.push(
                    self.error(CompileErrKind::MissingSemicolon)
                );
                return Err(());
            }
            return Ok(());
        } else if self.match_token(TokenKind::If) {
            todo!();
        } else {
            self.expression_statement()?;
            if !self.match_token(TokenKind::Semicolon) {
                self.errs.push(
                    self.error(CompileErrKind::MissingSemicolon)
                );
                return Err(());
            }
            self.chunks.push(self.chunk(OpCode::Pop as u8));
            return Ok(());
        }
    }

    fn expression_statement(&mut self) -> Result<(), ()> {
        let expr = self.expression();

        let mut had_err = false;
        for node in expr.dfs_postorder() {
            match node {
                ExpressionTreeNode::Branch(branch) => {
                    match branch {
                        ExpressionBranch::Operator(op) => {
                            for code in op.to_bytecodes() {
                                self.chunks.push(self.chunk(code as u8));
                            }
                        }
                    }
                }
                ExpressionTreeNode::Leaf(leaf) => {
                    match leaf {
                        ExpressionLeaf::Value(value) => {
                            self.chunks.push(self.chunk(OpCode::Constant as u8));
                            if self.values.len() == self.values.capacity() {
                                self.errs.push(self.error(CompileErrKind::TooManyValues))
                            } else {
                                self.chunks.push(self.chunk(self.values.len() as u8));
                                // Already checked the u8::MAX condition, so this should be infallible
                                self.values.push(value)
                                    .expect("fixed vec to not overflow after checking condition");
                            }
                        }
                        ExpressionLeaf::Error(err) => {
                            self.errs.push(self.error(CompileErrKind::UnexpectedToken(err)));
                            had_err = true;
                        }
                        other => {
                            println!("{:?}", other);
                            todo!();
                        }
                    }
                }
            }
        }
        
        if had_err {
            return Err(());
        } else {
            return Ok(());
        }
    }

    fn expression(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        self.assignment()
        // TODO
    }

    fn assignment(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        self.or()
        // TODO
    }

    fn or(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        let mut current = self.and();

        while self.match_token(TokenKind::Or) {
            current = BinaryTreeNode { 
                value: ExpressionTreeNode::Branch(ExpressionBranch::Operator(Operator::Or)),
                left: Some(Box::new(current)),
                right: Some(Box::new(self.and())),
            }
        }

        current
    }

    fn and(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        let mut current = self.equality();

        while self.match_token(TokenKind::And) {
            current = BinaryTreeNode { 
                value: ExpressionTreeNode::Branch(
                    ExpressionBranch::Operator(Operator::And)
                ),
                left: Some(Box::new(current)),
                right: Some(Box::new(self.equality())),
            }
        }

        current
    }

    fn equality(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        let mut current = self.comparison();

        loop {
            if self.match_token(TokenKind::EqualEqual) {
                current = BinaryTreeNode { 
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Equal)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.comparison())),
                };
            } else if self.match_token(TokenKind::BangEqual) {
                current = BinaryTreeNode { 
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Equal)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.comparison())),
                };

                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Not)
                    ),
                    left: None,
                    right: Some(Box::new(current))
                };
            } else {
                return current;
            }
        }
    }
    
    fn comparison(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        let mut current = self.term();

        loop {
            if self.match_token(TokenKind::Greater) {
                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Greater)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.term()))
                };
            } else if self.match_token(TokenKind::GreaterEqual) {
                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Less)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.term()))
                };

                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Not)
                    ),
                    left: None,
                    right: Some(Box::new(current))
                };
            } else if self.match_token(TokenKind::Less) {
                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Less)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.term()))
                };
            } else if self.match_token(TokenKind::LessEqual) {
                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Greater)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.term()))
                };

                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Not)
                    ),
                    left: None,
                    right: Some(Box::new(current))
                };
            }
            else {
                return current;
            }
        }
    }

    fn term(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        let mut current = self.factor();

        loop {
            if self.match_token(TokenKind::Minus) {
                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Subtract)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.factor()))
                };
            } else if self.match_token(TokenKind::Plus) {
                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Add)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.factor()))
                };
            } else {
                return current;
            }
        }
    }

    fn factor(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        let mut current = self.unary();

        loop {
            if self.match_token(TokenKind::Star) {
                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Multiply)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.unary()))
                };
            } else if self.match_token(TokenKind::Slash) {
                current = BinaryTreeNode {
                    value: ExpressionTreeNode::Branch(
                        ExpressionBranch::Operator(Operator::Divide)
                    ),
                    left: Some(Box::new(current)),
                    right: Some(Box::new(self.unary()))
                };
            } else {
                return current;
            }
        }
    }

    fn unary(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        if self.match_token(TokenKind::Bang) {
            return BinaryTreeNode {
                value: ExpressionTreeNode::Branch(
                    ExpressionBranch::Operator(Operator::Not)
                ),
                left: None,
                right: Some(Box::new(self.unary()))
            };
        } else if self.match_token(TokenKind::Minus) {
            return BinaryTreeNode {
                value: ExpressionTreeNode::Branch(
                    ExpressionBranch::Operator(Operator::SignFlip)
                ),
                left: None,
                right: Some(Box::new(self.unary()))
            };
        } else {
            return self.call();
        }
    }

    fn call(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        self.primary()
        // TODO
    }
    
    fn primary(&mut self) -> BinaryTreeNode<ExpressionTreeNode> {
        let token = self.next_token();
        match token {
            None => BinaryTreeNode {
                value: ExpressionTreeNode::Leaf(
                    ExpressionLeaf::Error(Unexpected { 
                        expected: vec![
                            TokenKind::True,
                            TokenKind::False,
                            TokenKind::Number,
                            TokenKind::String,
                            TokenKind::Nil,
                        ], 
                        actual: TokenKind::EOF, 
                        location: match self.previous {
                            None => Position::default(),
                            Some(token) => token.get_start()
                        }
                    })
                ),
                left: None,
                right: None,
            },
            Some(token) => {
                match token.kind() {
                    TokenKind::True => Self::value_node(Value::Boolean(true)),
                    TokenKind::False => Self::value_node(Value::Boolean(false)),
                    TokenKind::Number => Self::value_node(
                        Value::Number(
                            self.source_code[token.range()].parse::<f64>().expect("Number to successfully parse to f64")
                        )
                    ),
                    TokenKind::String => {
                        let source = &self.source_code[token.range()];
                        // Can't reference the source code because we want to free that string before runtime.
                        // Instead, clone it.
                        let source = source[1..source.len() - 1].to_string();
                        Self::value_node(
                        Value::Object(Rc::new(
                            Object::String(
                                source.into()
                            ))
                        ))
                    }
                    TokenKind::Nil => Self::value_node(Value::Nil),
                    TokenKind::LeftParen => {
                        // logical groupings reset to lowest precedence level
                        let result = self.or();
                        if !self.match_token(TokenKind::RightParen) {
                            return BinaryTreeNode {
                                value: ExpressionTreeNode::Leaf(
                                    ExpressionLeaf::Error(Unexpected { 
                                        expected: vec![
                                            TokenKind::RightParen
                                        ], 
                                        actual: match self.tokens
                                                .as_mut()
                                                .expect("tokenizer to be in active compiler")
                                                .peek() 
                                        {
                                            None => TokenKind::EOF,
                                            Some(token) => token.kind()
                                        }, 
                                        location: match self.previous {
                                            None => Position::default(),
                                            Some(token) => token.get_start()
                                        }
                                    })
                                ),
                                left: None,
                                right: None,
                            };
                        }
                        result
                    }
                    other => {
                        BinaryTreeNode {
                            value: ExpressionTreeNode::Leaf(
                                ExpressionLeaf::Error(Unexpected { 
                                    expected: vec![
                                        TokenKind::True,
                                        TokenKind::False,
                                        TokenKind::Number,
                                        TokenKind::String,
                                        TokenKind::Nil,
                                    ], 
                                    actual: other, 
                                    location: match self.previous {
                                        None => Position::default(),
                                        Some(token) => token.get_start()
                                    }
                                })
                            ),
                            left: None,
                            right: None,
                        }
                    }
                }
            }
        }
    }

    fn error_at_current(&self, kind: CompileErrKind) -> CompileErr {
        CompileErr {
            kind,
            location: self.current
                .expect("current to have a value when reporting errors")
                .get_start(),
        }
    }

    fn error(&self, kind: CompileErrKind) -> CompileErr {
        CompileErr { 
            kind, 
            location: self.location()
        }
    }

    fn chunk(&self, code: u8) -> Chunk {
        Chunk {
            line: self.location().line(),
            op: code,
        }
    }

    fn location(&self) -> Position {
        match self.previous {
            None => Position::default(),
            Some(token) => token.get_start()
        }
    }

    fn panic_mode_recovery(&mut self) {
        self.match_tokens_while(|token| {
            match token.kind() {
                // These signal the start of a statement. 
                // Break the loop if we find one
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::Print
                | TokenKind::Return => false,
                _ => true,
            }
        });
    }

    fn match_tokens_while<P: FnMut(&LoxToken) -> bool>(&mut self, mut predicate: P) {
        while self.match_token_if(&mut predicate) {}
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        self.match_token_if(|token| token.kind() == kind)
    }

    fn match_token_if<P: FnMut(&LoxToken) -> bool>(&mut self, mut predicate: P) -> bool {
        let tokens = self.tokens
            .as_mut()
            .expect("tokenizer to be in active compiler");

        match tokens.peek() {
            None => false,
            Some(token) => {
                if predicate(token) {
                    self.next_token().expect("Next to be Some() since we already peeked a Some() variant.");
                    true
                } else {
                    false
                }
            }
        }
    }

    fn next_token(&mut self) -> Option<LoxToken> {
        self.previous = self.current;
        self.current = self.tokens
            .as_mut()
            .expect("tokenizer to be in active compiler")
            .next();

        self.current
    }

    fn value_node(value: Value) -> BinaryTreeNode<ExpressionTreeNode> {
        BinaryTreeNode { 
            value: ExpressionTreeNode::Leaf(
                ExpressionLeaf::Value(value)
            ),
            left: None,
            right: None
        }
    }
}

enum CompilerState {
    Panic,
    Declaration,
}

struct Local {
    name: LoxToken,
    depth: usize,
    is_captured: bool,
}

enum FunctionType {
    Function,
    Initializer,
    Method,
    Script,
}

pub (crate) enum ErrKind {
    Parse(LoxParseErr),
    Compile(CompileErr),
}

#[derive(Clone, Debug)]
pub (crate) struct CompileErr {
    kind: CompileErrKind,
    location: Position,
}

impl Error for CompileErr {}
impl Display for CompileErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] Error: {:?}", self.location, self.kind)
    }
}

#[repr(u8)]
#[derive(Clone, Debug)]
pub (crate) enum CompileErrKind {
    Parse(LoxParseErr),
    UnexpectedToken(Unexpected),
    TooManyValues,
    MissingSemicolon,
}

#[derive(Clone, Debug)]
pub (crate) struct Unexpected {
    expected: Vec<TokenKind>,
    actual: TokenKind,
    location: Position,
}

impl Display for Unexpected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut expected_str = String::new();
        let last = self.expected.len() - 1;
        for (i, piece) in self.expected.iter().enumerate() {
            if i != 0 {
                if i == last {
                    expected_str.push_str(", or ")
                } else {
                    expected_str.push_str(", ");
                }
            }
            expected_str.push_str(&format!("{:?}", piece));
        }
        
        write!(f, "Unexpected token. Found {:?}, but expected: {}", self.actual, expected_str)
    }
}
impl Error for Unexpected {}

impl CompileErrKind {
    fn as_str(&self) -> &'static str {
        ""
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
enum Precedence {
    None = 0,
    Assignment = 1,  // =
    Or = 2,          // or
    And = 3,         // and
    Equality = 4,    // == !=
    Comparison = 5,  // < > <= >=
    Term = 6,        // + -
    Factor = 7,      // * /
    Unary = 8,       // ! -
    Call = 9,        // . ()
    Primary = 10
}

impl TryFrom<u8> for Precedence {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Precedence::None),
            1 => Ok(Precedence::Assignment),
            2 => Ok(Precedence::Or),
            3 => Ok(Precedence::And),
            4 => Ok(Precedence::Equality),
            5 => Ok(Precedence::Comparison),
            6 => Ok(Precedence::Term),
            7 => Ok(Precedence::Factor),
            8 => Ok(Precedence::Unary),
            9 => Ok(Precedence::Call),
            10 => Ok(Precedence::Primary),
            other => Err(other)
        }
    }
}

enum Statement {
    Expression(BinaryTreeNode<ExpressionTreeNode>),
}

trait BinaryTreeNodeExtensions {
    fn to_string(&self) -> String;
    fn to_string_helper(&self) -> (usize, String);
}

#[derive(Debug)]
enum ExpressionTreeNode {
    Branch(ExpressionBranch),
    Leaf(ExpressionLeaf)
}

enum ExpressionBranch {
    Operator(Operator),
}

impl Debug for ExpressionBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Operator(inner) => {
                Debug::fmt(inner, f)
            }
        }
    }
}

#[derive(Debug)]
enum ExpressionLeaf {
    Value(Value),
    Call,
    /// This indicates a syntax error
    Error(Unexpected),
}

#[derive(Debug)]
enum Operator {
    Assignment,
    Or,
    And,
    Equal,
    Not,
    Greater,
    Less,
    Divide,
    Multiply,
    Add,
    Subtract,
    SignFlip,
}

impl Operator {
    fn to_bytecodes(&self) -> impl Iterator<Item = OpCode> {
        match self {
            Operator::Assignment => todo!(),
            Operator::And => todo!(),
            Operator::Or => todo!(),
            Operator::Equal => [OpCode::Equal].into_iter(),
            Operator::Not => [OpCode::Not].into_iter(),
            Operator::Greater => [OpCode::Greater].into_iter(),
            Operator::Less => [OpCode::Less].into_iter(),
            Operator::Divide => [OpCode::Divide].into_iter(),
            Operator::Multiply => [OpCode::Multiply].into_iter(),
            Operator::Add => [OpCode::Add].into_iter(),
            Operator::Subtract => [OpCode::Subtract].into_iter(),
            Operator::SignFlip => [OpCode::Negate].into_iter(),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Operator::Assignment => "=",
            Operator::Or => "||",
            Operator::And => "&&",
            Operator::Equal => "==",
            Operator::Not => " !",
            Operator::Greater => " >",
            Operator::Less => " <",
            Operator::Divide => " /",
            Operator::Multiply => " *",
            Operator::Add => " +",
            Operator::Subtract => " -",
            Operator::SignFlip => " -",
        }
    }
}