use crate::assemble::lex::Token;

#[derive(Debug)]
pub struct Program {
    pub function_def: FunctionDef,
}

#[derive(Debug)]
pub struct FunctionDef {
    pub name: String,
    pub body: Vec<BlockItem>,
}

#[derive(Debug)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

#[derive(Debug)]
pub enum Statement {
    Return(Expression),
    Expression(Expression),
    Null,
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub init: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(u64),
    Var(String),
    Unary(UnaryOp, Box<Expression>),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Assignment(Box<Expression>, Box<Expression>),
    PrefixIncr(Box<Expression>),
    PrefixDecr(Box<Expression>),
    PostfixIncr(Box<Expression>),
    PostfixDecr(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Complement,
    Negation,
    Not,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    LeftShift,
    RightShift,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    And,
    Or,
    EqualTo,
    NotEqualTo,
    LessThan,
    GreaterThan,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
    Assignment,
    CmpdAddAssn,
    CmpdSubAssn,
    CmpdMulAssn,
    CmpdDivAssn,
    CmpdRemAssn,
    CmpdLShiftAssn,
    CmpdRShiftAssn,
    CmpdBitwiseAndAssn,
    CmpdBitwiseXorAssn,
    CmpdBitwiseOrAssn,
}

impl BinaryOp {
    fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Assignment
            | BinaryOp::CmpdAddAssn
            | BinaryOp::CmpdSubAssn
            | BinaryOp::CmpdMulAssn
            | BinaryOp::CmpdDivAssn
            | BinaryOp::CmpdRemAssn
            | BinaryOp::CmpdLShiftAssn
            | BinaryOp::CmpdRShiftAssn
            | BinaryOp::CmpdBitwiseAndAssn
            | BinaryOp::CmpdBitwiseXorAssn
            | BinaryOp::CmpdBitwiseOrAssn => 1,
            BinaryOp::Or => 5,
            BinaryOp::And => 10,
            BinaryOp::BitwiseOr => 15,
            BinaryOp::BitwiseXor => 20,
            BinaryOp::BitwiseAnd => 25,
            BinaryOp::EqualTo | BinaryOp::NotEqualTo => 30,
            BinaryOp::LessThan
            | BinaryOp::GreaterThan
            | BinaryOp::LessThanOrEqualTo
            | BinaryOp::GreaterThanOrEqualTo => 35,
            BinaryOp::LeftShift | BinaryOp::RightShift => 40,
            BinaryOp::Add | BinaryOp::Subtract => 45,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Remainder => 50,
        }
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    fn parse_program(&mut self) -> anyhow::Result<Program> {
        let function_def = self.parse_function_def()?;

        match self.next() {
            None => {}
            Some(other) => {
                anyhow::bail!("Invalid program: expected end of input, found {:?}", other);
            }
        }

        Ok(Program { function_def })
    }

    fn parse_function_def(&mut self) -> anyhow::Result<FunctionDef> {
        self.expect(Token::Int, "Invalid funtion defintion")?;

        let name = match self.next() {
            Some(Token::Identifier(val)) => val.clone(),
            Some(other) => anyhow::bail!(
                "Invalid function definition: expected identifier, found: {:?}",
                other
            ),
            None => anyhow::bail!("Invalid function definition: found end of input"),
        };

        self.expect(Token::OpenParen, "Invalid funtion defintion")?;
        self.expect(Token::Void, "Invalid funtion defintion")?;
        self.expect(Token::CloseParen, "Invalid funtion defintion")?;

        self.expect(Token::OpenBrace, "Invalid funtion defintion")?;
        let mut body = Vec::new();
        while self.peek() != Some(&Token::CloseBrace) {
            let next_block_item = self.parse_block_item()?;
            body.push(next_block_item);
        }
        self.expect(Token::CloseBrace, "Invalid funtion defintion")?;

        Ok(FunctionDef { name, body })
    }

    fn parse_block_item(&mut self) -> anyhow::Result<BlockItem> {
        match self.peek() {
            Some(&Token::Int) => Ok(BlockItem::D(self.parse_declaration()?)),
            _ => Ok(BlockItem::S(self.parse_statement()?)),
        }
    }

    fn parse_statement(&mut self) -> anyhow::Result<Statement> {
        match self.peek() {
            Some(&Token::Return) => {
                self.next();
                let exp = self.parse_expression(0)?;
                self.expect(Token::Semicolon, "Invalid statement")?;
                Ok(Statement::Return(exp))
            }
            Some(&Token::Semicolon) => {
                self.next();
                Ok(Statement::Null)
            }
            _ => {
                let exp = self.parse_expression(0)?;
                self.expect(Token::Semicolon, "Invalid statement")?;
                Ok(Statement::Expression(exp))
            }
        }
    }

    fn parse_declaration(&mut self) -> anyhow::Result<Declaration> {
        self.expect(Token::Int, "Invalid declaration, unsupported data type")?;

        let name = match self.next() {
            Some(Token::Identifier(val)) => val.clone(),
            Some(other) => anyhow::bail!(
                "Invalid declaration: expected identifier, found: {:?}",
                other
            ),
            None => anyhow::bail!("Invalid declaration: found end of input"),
        };

        let init = match self.peek() {
            Some(&Token::Assignment) => {
                self.next();
                Some(self.parse_expression(0)?)
            }
            _ => None,
        };

        self.expect(Token::Semicolon, "Invalid declaration")?;

        Ok(Declaration { name, init })
    }

    fn parse_expression(&mut self, min_prec: u8) -> anyhow::Result<Expression> {
        let mut left = self.parse_factor()?;
        while let Some(tok) = self.peek() {
            match Self::parse_binary_op(tok) {
                Ok(op) if op.precedence() >= min_prec => {
                    self.next();
                    match Self::parse_cmpd_assn_op(&op) {
                        Some(inner_op) => {
                            let exp = self.parse_expression(op.precedence())?;
                            let right =
                                Expression::Binary(inner_op, Box::new(left.clone()), Box::new(exp));
                            left = Expression::Assignment(Box::new(left), Box::new(right));
                        }
                        None => match op {
                            BinaryOp::Assignment => {
                                let right = self.parse_expression(op.precedence())?;
                                left = Expression::Assignment(Box::new(left), Box::new(right));
                            }
                            _ => {
                                let right = self.parse_expression(op.precedence() + 1)?;
                                left = Expression::Binary(op, Box::new(left), Box::new(right));
                            }
                        },
                    }
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> anyhow::Result<Expression> {
        let factor = self.parse_primary_factor()?;
        match self.peek() {
            Some(Token::Increment) => {
                self.next();
                Ok(Expression::PostfixIncr(Box::new(factor)))
            }
            Some(Token::Decrement) => {
                self.next();
                Ok(Expression::PostfixDecr(Box::new(factor)))
            }
            _ => Ok(factor),
        }
    }

    fn parse_primary_factor(&mut self) -> anyhow::Result<Expression> {
        match self.next() {
            Some(Token::Constant(val)) => Ok(Expression::Constant(*val)),
            Some(Token::Identifier(val)) => Ok(Expression::Var(val.clone())),
            Some(tok @ (Token::Complement | Token::Negation | Token::Not)) => {
                let op = Self::parse_unary_op(tok)?;
                let exp = self.parse_factor()?;
                Ok(Expression::Unary(op, Box::new(exp)))
            }
            Some(Token::Increment) => {
                let factor = self.parse_factor()?;
                Ok(Expression::PrefixIncr(Box::new(factor)))
            }
            Some(Token::Decrement) => {
                let factor = self.parse_factor()?;
                Ok(Expression::PrefixDecr(Box::new(factor)))
            }
            Some(Token::OpenParen) => {
                let exp = self.parse_expression(0)?;
                self.expect(Token::CloseParen, "Invalid expression")?;
                Ok(exp)
            }
            Some(other) => {
                anyhow::bail!(
                    "Invalid expression: expected a constant or unary op, found {:?}",
                    other
                )
            }
            None => anyhow::bail!("Invalid expression: expected expression, found end of input"),
        }
    }

    fn parse_unary_op(tok: &Token) -> anyhow::Result<UnaryOp> {
        match tok {
            Token::Complement => Ok(UnaryOp::Complement),
            Token::Negation => Ok(UnaryOp::Negation),
            Token::Not => Ok(UnaryOp::Not),
            _ => anyhow::bail!("Invalid unary operator"),
        }
    }

    fn parse_binary_op(tok: &Token) -> anyhow::Result<BinaryOp> {
        match tok {
            Token::Addition => Ok(BinaryOp::Add),
            Token::Negation => Ok(BinaryOp::Subtract),
            Token::Multiplication => Ok(BinaryOp::Multiply),
            Token::Division => Ok(BinaryOp::Divide),
            Token::Remainder => Ok(BinaryOp::Remainder),
            Token::LeftShift => Ok(BinaryOp::LeftShift),
            Token::RightShift => Ok(BinaryOp::RightShift),
            Token::BitwiseAnd => Ok(BinaryOp::BitwiseAnd),
            Token::BitwiseXor => Ok(BinaryOp::BitwiseXor),
            Token::BitwiseOr => Ok(BinaryOp::BitwiseOr),
            Token::And => Ok(BinaryOp::And),
            Token::Or => Ok(BinaryOp::Or),
            Token::EqualTo => Ok(BinaryOp::EqualTo),
            Token::NotEqualTo => Ok(BinaryOp::NotEqualTo),
            Token::LessThan => Ok(BinaryOp::LessThan),
            Token::GreaterThan => Ok(BinaryOp::GreaterThan),
            Token::LessThanOrEqualTo => Ok(BinaryOp::LessThanOrEqualTo),
            Token::GreaterThanOrEqualTo => Ok(BinaryOp::GreaterThanOrEqualTo),
            Token::Assignment => Ok(BinaryOp::Assignment),
            Token::CmpdAddAssn => Ok(BinaryOp::CmpdAddAssn),
            Token::CmpdSubAssn => Ok(BinaryOp::CmpdSubAssn),
            Token::CmpdMulAssn => Ok(BinaryOp::CmpdMulAssn),
            Token::CmpdDivAssn => Ok(BinaryOp::CmpdDivAssn),
            Token::CmpdRemAssn => Ok(BinaryOp::CmpdRemAssn),
            Token::CmpdLShiftAssn => Ok(BinaryOp::CmpdLShiftAssn),
            Token::CmpdRShiftAssn => Ok(BinaryOp::CmpdRShiftAssn),
            Token::CmpdBitwiseAndAssn => Ok(BinaryOp::CmpdBitwiseAndAssn),
            Token::CmpdBitwiseXorAssn => Ok(BinaryOp::CmpdBitwiseXorAssn),
            Token::CmpdBitwiseOrAssn => Ok(BinaryOp::CmpdBitwiseOrAssn),
            _ => anyhow::bail!("Invalid binary operator"),
        }
    }

    fn parse_cmpd_assn_op(op: &BinaryOp) -> Option<BinaryOp> {
        match op {
            BinaryOp::CmpdAddAssn => Some(BinaryOp::Add),
            BinaryOp::CmpdSubAssn => Some(BinaryOp::Subtract),
            BinaryOp::CmpdMulAssn => Some(BinaryOp::Multiply),
            BinaryOp::CmpdDivAssn => Some(BinaryOp::Divide),
            BinaryOp::CmpdRemAssn => Some(BinaryOp::Remainder),
            BinaryOp::CmpdLShiftAssn => Some(BinaryOp::LeftShift),
            BinaryOp::CmpdRShiftAssn => Some(BinaryOp::RightShift),
            BinaryOp::CmpdBitwiseAndAssn => Some(BinaryOp::BitwiseAnd),
            BinaryOp::CmpdBitwiseXorAssn => Some(BinaryOp::BitwiseXor),
            BinaryOp::CmpdBitwiseOrAssn => Some(BinaryOp::BitwiseOr),
            _ => None,
        }
    }

    fn expect(&mut self, expected: Token, context: &str) -> anyhow::Result<()> {
        match self.next() {
            Some(tok) if *tok == expected => Ok(()),
            Some(other) => anyhow::bail!("{}: expected {:?}, found {:?}", context, expected, other),
            None => anyhow::bail!("{}: expected {:?}, found end of input", context, expected),
        }
    }
}

pub fn parse(tokens: &[Token]) -> anyhow::Result<Program> {
    let mut parser = Parser { tokens, pos: 0 };
    parser.parse_program()
}
