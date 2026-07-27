use crate::assemble::lex::Token;

pub struct Program {
    pub function_def: FunctionDef,
}

impl Program {
    pub fn print_program(&self) {
        println!("Program(");
        println!("\tFunction(");
        println!("\t\tname=\"{}\"", self.function_def.name);
        println!("\t\tbody=Return(");
        let Statement::Return(Expression::Constant(constant)) = self.function_def.body;
        println!("\t\t\tConstant({})", constant);
        println!("\t\t)");
        println!("\t)");
        println!(")");
    }
}

pub struct FunctionDef {
    pub name: String,
    pub body: Statement,
}

pub enum Statement {
    Return(Expression),
}

pub enum Expression {
    Constant(u64),
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
        let exp = self.parse_expression()?;
        self.expect(Token::Semicolon, "Invalid statement")?;

        Ok(Statement::Return(exp))
    }

    fn parse_expression(&mut self) -> anyhow::Result<Expression> {
        match self.next() {
            Some(Token::Constant(val)) => Ok(Expression::Constant(*val)),
            Some(other) => {
                anyhow::bail!("Invalid expression: expected a constant, found {:?}", other)
            }
            None => anyhow::bail!("Invalid expression: expected expression, found end of input"),
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
