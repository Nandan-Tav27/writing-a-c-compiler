use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Constant(u64),
    Int,
    Void,
    Return,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Complement,
    Negation,
    Decrement,
    Addition,
    Multiplication,
    Division,
    Remainder,
    LeftShift,
    RightShift,
    BitwiseAND,
    BitwiseXOR,
    BitwiseOR,
}

pub fn lex(file_path: &Path) -> anyhow::Result<Vec<Token>> {
    let contents = std::fs::read_to_string(file_path)?;
    let mut chars = contents.chars().peekable();

    // Iterate through chars and generate a Vec<Token>
    let mut tokens: Vec<Token> = Vec::new();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut val: Vec<char> = Vec::new();
                val.push(chars.next().unwrap());
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        val.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                let value = val.into_iter().collect::<String>();
                let token = match value.as_str() {
                    "int" => Token::Int,
                    "void" => Token::Void,
                    "return" => Token::Return,
                    _ => Token::Identifier(value),
                };
                tokens.push(token);
            }
            c if c.is_ascii_digit() => {
                let mut val: Vec<char> = Vec::new();
                val.push(chars.next().unwrap());
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        val.push(chars.next().unwrap());
                    } else if c.is_ascii_alphabetic() {
                        val.push(chars.next().unwrap());
                        let value = val.into_iter().collect::<String>();
                        anyhow::bail!("Invalid constant: {:?}...", value);
                    } else {
                        break;
                    }
                }
                let value = val.into_iter().collect::<String>().parse::<u64>()?;
                tokens.push(Token::Constant(value));
            }
            '-' => {
                chars.next();
                if chars.peek() == Some(&'-') {
                    tokens.push(Token::Decrement);
                    chars.next();
                } else {
                    tokens.push(Token::Negation);
                }
            }
            '+' => {
                tokens.push(Token::Addition);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Multiplication);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Division);
                chars.next();
            }
            '%' => {
                tokens.push(Token::Remainder);
                chars.next();
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'<') {
                    tokens.push(Token::LeftShift);
                    chars.next();
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    tokens.push(Token::RightShift);
                    chars.next();
                }
            }
            '&' => {
                tokens.push(Token::BitwiseAND);
                chars.next();
            }
            '^' => {
                tokens.push(Token::BitwiseXOR);
                chars.next();
            }
            '|' => {
                tokens.push(Token::BitwiseOR);
                chars.next();
            }
            '~' => {
                tokens.push(Token::Complement);
                chars.next();
            }
            '(' => {
                tokens.push(Token::OpenParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::CloseParen);
                chars.next();
            }
            '{' => {
                tokens.push(Token::OpenBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::CloseBrace);
                chars.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            _ => {
                anyhow::bail!("Invalid character: {:?}", c);
            }
        }
    }

    Ok(tokens)
}
