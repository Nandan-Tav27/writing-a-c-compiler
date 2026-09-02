use crate::assemble::lex::Token;

#[derive(Debug)]
pub struct Program {
    pub function_def: FunctionDef,
}

#[derive(Debug)]
pub struct FunctionDef {
    pub name: String,
    pub body: Statement,
}

#[derive(Debug)]
pub enum Statement {
    Return(Expression),
}

#[derive(Debug)]
pub enum Expression {
    Constant(u64),
    Unary(UnaryOp, Box<Expression>),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
}

#[derive(Debug)]
pub enum UnaryOp {
    Complement,
    Negation,
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    LeftShift,
    RightShift,
    BitwiseAND,
    BitwiseXOR,
    BitwiseOR,
}

impl BinaryOp {
    fn precedence(&self) -> u8 {
        match self {
            BinaryOp::BitwiseOR => 25,
            BinaryOp::BitwiseXOR => 30,
            BinaryOp::BitwiseAND => 35,
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
        let body = self.parse_statement()?;
        self.expect(Token::CloseBrace, "Invalid funtion defintion")?;

        Ok(FunctionDef { name, body })
    }

    fn parse_statement(&mut self) -> anyhow::Result<Statement> {
        self.expect(Token::Return, "Invalid statement")?;
        let exp = self.parse_expression(0)?;
        self.expect(Token::Semicolon, "Invalid statement")?;

        Ok(Statement::Return(exp))
    }

    fn parse_expression(&mut self, min_prec: u8) -> anyhow::Result<Expression> {
        let mut left = self.parse_factor()?;
        while let Some(tok) = self.peek() {
            match Self::parse_binary_op(tok) {
                Ok(op) if op.precedence() >= min_prec => {
                    self.next();
                    let right = self.parse_expression(op.precedence() + 1)?;
                    left = Expression::Binary(op, Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> anyhow::Result<Expression> {
        match self.next() {
            Some(Token::Constant(val)) => Ok(Expression::Constant(*val)),
            Some(tok @ (Token::Complement | Token::Negation)) => {
                let op = Self::parse_unary_op(tok)?;
                let exp = self.parse_factor()?;
                Ok(Expression::Unary(op, Box::new(exp)))
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
            Token::BitwiseAND => Ok(BinaryOp::BitwiseAND),
            Token::BitwiseXOR => Ok(BinaryOp::BitwiseXOR),
            Token::BitwiseOR => Ok(BinaryOp::BitwiseOR),
            _ => anyhow::bail!("Invalid binary operator"),
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
