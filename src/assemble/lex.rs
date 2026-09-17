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
    Increment,
    Decrement,
    Addition,
    Multiplication,
    Division,
    Remainder,
    LeftShift,
    RightShift,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    Not,
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
                match chars.peek() {
                    Some(&'-') => {
                        chars.next();
                        tokens.push(Token::Decrement);
                    }
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::CmpdSubAssn);
                    }
                    _ => tokens.push(Token::Negation),
                }
            }
            '+' => {
                chars.next();
                match chars.peek() {
                    Some(&'+') => {
                        chars.next();
                        tokens.push(Token::Increment);
                    }
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::CmpdAddAssn);
                    }
                    _ => tokens.push(Token::Addition),
                }
            }
            '*' => {
                chars.next();
                match chars.peek() {
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::CmpdMulAssn);
                    }
                    _ => tokens.push(Token::Multiplication),
                }
            }
            '/' => {
                chars.next();
                match chars.peek() {
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::CmpdDivAssn);
                    }
                    _ => tokens.push(Token::Division),
                }
            }
            '%' => {
                chars.next();
                match chars.peek() {
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::CmpdRemAssn);
                    }
                    _ => tokens.push(Token::Remainder),
                }
            }
            '<' => {
                chars.next();
                match chars.peek() {
                    Some(&'<') => {
                        chars.next();
                        match chars.peek() {
                            Some(&'=') => {
                                chars.next();
                                tokens.push(Token::CmpdLShiftAssn);
                            }
                            _ => tokens.push(Token::LeftShift),
                        }
                    }
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::LessThanOrEqualTo);
                    }
                    _ => tokens.push(Token::LessThan),
                }
            }
            '>' => {
                chars.next();
                match chars.peek() {
                    Some(&'>') => {
                        chars.next();
                        match chars.peek() {
                            Some(&'=') => {
                                chars.next();
                                tokens.push(Token::CmpdRShiftAssn);
                            }
                            _ => tokens.push(Token::RightShift),
                        }
                    }
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::GreaterThanOrEqualTo);
                    }
                    _ => tokens.push(Token::GreaterThan),
                }
            }
            '&' => {
                chars.next();
                match chars.peek() {
                    Some(&'&') => {
                        chars.next();
                        tokens.push(Token::And);
                    }
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::CmpdBitwiseAndAssn);
                    }
                    _ => tokens.push(Token::BitwiseAnd),
                }
            }
            '^' => {
                chars.next();
                match chars.peek() {
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::CmpdBitwiseXorAssn);
                    }
                    _ => tokens.push(Token::BitwiseXor),
                }
            }
            '|' => {
                chars.next();
                match chars.peek() {
                    Some(&'|') => {
                        chars.next();
                        tokens.push(Token::Or);
                    }
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::CmpdBitwiseOrAssn);
                    }
                    _ => tokens.push(Token::BitwiseOr),
                }
            }
            '!' => {
                chars.next();
                match chars.peek() {
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::NotEqualTo);
                    }
                    _ => tokens.push(Token::Not),
                }
            }
            '~' => {
                chars.next();
                tokens.push(Token::Complement);
            }
            '=' => {
                chars.next();
                match chars.peek() {
                    Some(&'=') => {
                        chars.next();
                        tokens.push(Token::EqualTo);
                    }
                    _ => tokens.push(Token::Assignment),
                }
            }
            '(' => {
                chars.next();
                tokens.push(Token::OpenParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::CloseParen);
            }
            '{' => {
                chars.next();
                tokens.push(Token::OpenBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::CloseBrace);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            _ => {
                anyhow::bail!("Invalid character: {:?}", c);
            }
        }
    }

    Ok(tokens)
}
