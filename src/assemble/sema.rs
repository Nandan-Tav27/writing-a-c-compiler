use std::collections::HashMap;

use crate::assemble::parse::{self, Declaration};

struct Resolver {
    var_count: usize,
}

impl Resolver {
    fn resolve_program(&mut self, program: parse::Program) -> anyhow::Result<parse::Program> {
        let parse::FunctionDef { name, body } = program.function_def;
        let mut block_items = Vec::new();
        let mut variable_map: HashMap<String, String> = HashMap::new();
        for item in body {
            let resolved_item = match item {
                parse::BlockItem::S(statement) => {
                    parse::BlockItem::S(self.resolve_statement(statement, &mut variable_map)?)
                }
                parse::BlockItem::D(decalration) => {
                    parse::BlockItem::D(self.resolve_declaration(decalration, &mut variable_map)?)
                }
            };
            block_items.push(resolved_item);
        }
        Ok(parse::Program {
            function_def: parse::FunctionDef {
                name,
                body: block_items,
            },
        })
    }

    fn resolve_declaration(
        &mut self,
        declaration: parse::Declaration,
        variable_map: &mut HashMap<String, String>,
    ) -> anyhow::Result<parse::Declaration> {
        let Declaration { name, init } = declaration;
        if variable_map.contains_key(&name) {
            anyhow::bail!("Invalid declaration: variable is already defined");
        }
        let var_name = self.make_tmp(name.clone());
        variable_map.insert(name, var_name.clone());
        let init = match init {
            Some(exp) => Some(self.resolve_expression(exp, variable_map)?),
            _ => None,
        };
        Ok(Declaration {
            name: var_name,
            init,
        })
    }

    fn resolve_statement(
        &mut self,
        statement: parse::Statement,
        variable_map: &mut HashMap<String, String>,
    ) -> anyhow::Result<parse::Statement> {
        match statement {
            parse::Statement::Return(exp) => Ok(parse::Statement::Return(
                self.resolve_expression(exp, variable_map)?,
            )),
            parse::Statement::Expression(exp) => Ok(parse::Statement::Expression(
                self.resolve_expression(exp, variable_map)?,
            )),
            parse::Statement::Null => Ok(parse::Statement::Null),
        }
    }

    fn resolve_expression(
        &mut self,
        exp: parse::Expression,
        variable_map: &mut HashMap<String, String>,
    ) -> anyhow::Result<parse::Expression> {
        match exp {
            // NOTE: We check for valid lvalues here, for now...
            parse::Expression::Assignment(exp1, exp2) => {
                if let parse::Expression::Var(_) = *exp1 {
                    let exp1 = self.resolve_expression(*exp1, variable_map)?;
                    let exp2 = self.resolve_expression(*exp2, variable_map)?;
                    Ok(parse::Expression::Assignment(
                        Box::new(exp1),
                        Box::new(exp2),
                    ))
                } else {
                    anyhow::bail!("Invalid assignment expression: invalid lvalue")
                }
            }
            parse::Expression::Var(name) => match variable_map.get(&name) {
                Some(resolved_name) => Ok(parse::Expression::Var(resolved_name.clone())),
                None => anyhow::bail!("Invalid variable: undeclared"),
            },
            parse::Expression::Unary(op, exp) => Ok(parse::Expression::Unary(
                op,
                Box::new(self.resolve_expression(*exp, variable_map)?),
            )),
            parse::Expression::Binary(op, exp1, exp2) => Ok(parse::Expression::Binary(
                op,
                Box::new(self.resolve_expression(*exp1, variable_map)?),
                Box::new(self.resolve_expression(*exp2, variable_map)?),
            )),
            parse::Expression::PrefixIncr(exp) => {
                // NOTE: We check for valid lvalues here, for now...
                if let parse::Expression::Var(_) = *exp {
                    let exp = self.resolve_expression(*exp, variable_map)?;
                    Ok(parse::Expression::PrefixIncr(Box::new(exp)))
                } else {
                    anyhow::bail!("Invalid increment expression: invalid lvalue")
                }
            }
            parse::Expression::PrefixDecr(exp) => {
                // NOTE: We check for valid lvalues here, for now...
                if let parse::Expression::Var(_) = *exp {
                    let exp = self.resolve_expression(*exp, variable_map)?;
                    Ok(parse::Expression::PrefixDecr(Box::new(exp)))
                } else {
                    anyhow::bail!("Invalid decrement expression: invalid lvalue")
                }
            }
            parse::Expression::PostfixIncr(exp) => {
                // NOTE: We check for valid lvalues here, for now...
                if let parse::Expression::Var(_) = *exp {
                    let exp = self.resolve_expression(*exp, variable_map)?;
                    Ok(parse::Expression::PostfixIncr(Box::new(exp)))
                } else {
                    anyhow::bail!("Invalid increment expression: invalid lvalue")
                }
            }
            parse::Expression::PostfixDecr(exp) => {
                // NOTE: We check for valid lvalues here, for now...
                if let parse::Expression::Var(_) = *exp {
                    let exp = self.resolve_expression(*exp, variable_map)?;
                    Ok(parse::Expression::PostfixDecr(Box::new(exp)))
                } else {
                    anyhow::bail!("Invalid decrement expression: invalid lvalue")
                }
            }
            _ => Ok(exp),
        }
    }

    fn make_tmp(&mut self, name: String) -> String {
        let tmp_var = format!("{}.{}", name, self.var_count);
        self.var_count += 1;
        tmp_var
    }
}

pub fn resolve(
    program: parse::Program,
    tmp_var_count: &mut usize,
) -> anyhow::Result<parse::Program> {
    let var_count = *tmp_var_count;
    let mut resolver = Resolver { var_count };
    let program = resolver.resolve_program(program)?;
    *tmp_var_count = resolver.var_count;
    Ok(program)
}
