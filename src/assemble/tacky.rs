use crate::assemble::parse;

#[derive(Debug)]
pub struct Program {
    pub function_def: FunctionDef,
}

#[derive(Debug)]
pub struct FunctionDef {
    pub name: String,
    pub body: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    Ret(Value),
    // dest must be Value::Var
    Unary {
        unary_operator: UnaryOp,
        src: Value,
        dest: Value,
    },
    Binary {
        binary_operator: BinaryOp,
        src1: Value,
        src2: Value,
        dest: Value,
    },
    Copy {
        src: Value,
        dest: Value,
    },
    Jump {
        label: String,
    },
    JumpIfZero {
        condition: Value,
        label: String,
    },
    JumpIfNotZero {
        condition: Value,
        label: String,
    },
    Label(String),
}

#[derive(Debug)]
pub enum Value {
    Constant(u64),
    Var(String),
}

#[derive(Debug)]
pub enum UnaryOp {
    Complement,
    Negation,
    Not,
}

impl From<parse::UnaryOp> for UnaryOp {
    fn from(op: parse::UnaryOp) -> Self {
        match op {
            parse::UnaryOp::Complement => UnaryOp::Complement,
            parse::UnaryOp::Negation => UnaryOp::Negation,
            parse::UnaryOp::Not => UnaryOp::Not,
        }
    }
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
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    EqualTo,
    NotEqualTo,
    LessThan,
    GreaterThan,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
}

impl BinaryOp {
    fn parse_binary_op_to_tacky(op: parse::BinaryOp) -> anyhow::Result<BinaryOp> {
        match op {
            parse::BinaryOp::Add => Ok(BinaryOp::Add),
            parse::BinaryOp::Subtract => Ok(BinaryOp::Subtract),
            parse::BinaryOp::Multiply => Ok(BinaryOp::Multiply),
            parse::BinaryOp::Divide => Ok(BinaryOp::Divide),
            parse::BinaryOp::Remainder => Ok(BinaryOp::Remainder),
            parse::BinaryOp::LeftShift => Ok(BinaryOp::LeftShift),
            parse::BinaryOp::RightShift => Ok(BinaryOp::RightShift),
            parse::BinaryOp::BitwiseAnd => Ok(BinaryOp::BitwiseAnd),
            parse::BinaryOp::BitwiseXor => Ok(BinaryOp::BitwiseXor),
            parse::BinaryOp::BitwiseOr => Ok(BinaryOp::BitwiseOr),
            parse::BinaryOp::EqualTo => Ok(BinaryOp::EqualTo),
            parse::BinaryOp::NotEqualTo => Ok(BinaryOp::NotEqualTo),
            parse::BinaryOp::LessThan => Ok(BinaryOp::LessThan),
            parse::BinaryOp::GreaterThan => Ok(BinaryOp::GreaterThan),
            parse::BinaryOp::LessThanOrEqualTo => Ok(BinaryOp::LessThanOrEqualTo),
            parse::BinaryOp::GreaterThanOrEqualTo => Ok(BinaryOp::GreaterThanOrEqualTo),
            _ => anyhow::bail!("Invalid binary operator to tacky conversion"),
        }
    }
}

pub struct TackyTransformer {
    tmp_var_count: usize,
    and_false_label: usize,
    or_true_label: usize,
}

impl TackyTransformer {
    fn lower_program(&mut self, program: parse::Program) -> Program {
        let function_def = self.lower_function_def(program.function_def);
        Program { function_def }
    }

    fn lower_function_def(&mut self, function_def: parse::FunctionDef) -> FunctionDef {
        let parse::FunctionDef { name, body } = function_def;
        let mut instrs = Vec::new();
        for block_item in body {
            match block_item {
                parse::BlockItem::S(statement) => self.lower_statement(statement, &mut instrs),
                parse::BlockItem::D(declaration) => {
                    self.lower_declaration(declaration, &mut instrs)
                }
            }
        }
        instrs.push(Instruction::Ret(Value::Constant(0)));
        FunctionDef { name, body: instrs }
    }

    fn lower_statement(&mut self, statement: parse::Statement, instrs: &mut Vec<Instruction>) {
        match statement {
            parse::Statement::Return(exp) => {
                let val = self.lower_expression(exp, instrs);
                instrs.push(Instruction::Ret(val));
            }
            parse::Statement::Expression(exp) => {
                self.lower_expression(exp, instrs);
            }
            parse::Statement::Null => {}
        }
    }

    fn lower_declaration(
        &mut self,
        declaration: parse::Declaration,
        instrs: &mut Vec<Instruction>,
    ) {
        if let Some(init) = declaration.init {
            self.lower_expression(
                parse::Expression::Assignment(
                    Box::new(parse::Expression::Var(declaration.name)),
                    Box::new(init),
                ),
                instrs,
            );
        }
    }

    fn lower_expression(&mut self, exp: parse::Expression, instrs: &mut Vec<Instruction>) -> Value {
        match exp {
            parse::Expression::Constant(val) => Value::Constant(val),
            parse::Expression::Unary(op, exp) => {
                let unary_operator: UnaryOp = op.into();
                let src = self.lower_expression(*exp, instrs);
                let var = format!("tmp.{}", self.tmp_var_count);
                let dest = Value::Var(var.clone());
                self.tmp_var_count += 1;
                let instr = Instruction::Unary {
                    unary_operator,
                    src,
                    dest,
                };
                instrs.push(instr);
                Value::Var(var)
            }
            parse::Expression::Binary(parse::BinaryOp::And, exp1, exp2) => {
                let var = format!("tmp.{}", self.tmp_var_count);
                self.tmp_var_count += 1;
                let label = format!("and_false{}", self.and_false_label);
                let end_label = format!("and_false_end{}", self.and_false_label);
                self.and_false_label += 1;

                let src1 = self.lower_expression(*exp1, instrs);
                instrs.push(Instruction::JumpIfZero {
                    condition: src1,
                    label: label.clone(),
                });

                let src2 = self.lower_expression(*exp2, instrs);
                instrs.push(Instruction::JumpIfZero {
                    condition: src2,
                    label: label.clone(),
                });

                instrs.push(Instruction::Copy {
                    src: Value::Constant(1),
                    dest: Value::Var(var.clone()),
                });
                instrs.push(Instruction::Jump {
                    label: end_label.clone(),
                });

                instrs.push(Instruction::Label(label));
                instrs.push(Instruction::Copy {
                    src: Value::Constant(0),
                    dest: Value::Var(var.clone()),
                });
                instrs.push(Instruction::Label(end_label));

                Value::Var(var)
            }
            parse::Expression::Binary(parse::BinaryOp::Or, exp1, exp2) => {
                let var = format!("tmp.{}", self.tmp_var_count);
                self.tmp_var_count += 1;
                let label = format!("or_true{}", self.or_true_label);
                let end_label = format!("or_true_end{}", self.or_true_label);
                self.or_true_label += 1;

                let src1 = self.lower_expression(*exp1, instrs);
                instrs.push(Instruction::JumpIfNotZero {
                    condition: src1,
                    label: label.clone(),
                });

                let src2 = self.lower_expression(*exp2, instrs);
                instrs.push(Instruction::JumpIfNotZero {
                    condition: src2,
                    label: label.clone(),
                });

                instrs.push(Instruction::Copy {
                    src: Value::Constant(0),
                    dest: Value::Var(var.clone()),
                });
                instrs.push(Instruction::Jump {
                    label: end_label.clone(),
                });

                instrs.push(Instruction::Label(label));
                instrs.push(Instruction::Copy {
                    src: Value::Constant(1),
                    dest: Value::Var(var.clone()),
                });
                instrs.push(Instruction::Label(end_label));

                Value::Var(var)
            }
            parse::Expression::Binary(op, exp1, exp2) => {
                let binary_operator: BinaryOp = BinaryOp::parse_binary_op_to_tacky(op).unwrap();
                let src1 = self.lower_expression(*exp1, instrs);
                let src2 = self.lower_expression(*exp2, instrs);
                let var = format!("tmp.{}", self.tmp_var_count);
                let dest = Value::Var(var.clone());
                self.tmp_var_count += 1;
                let instr = Instruction::Binary {
                    binary_operator,
                    src1,
                    src2,
                    dest,
                };
                instrs.push(instr);
                Value::Var(var)
            }
            parse::Expression::Var(name) => Value::Var(name),
            parse::Expression::Assignment(exp1, exp2) => {
                let parse::Expression::Var(v) = *exp1 else {
                    unreachable!(
                        "Assignment lvalue must be a Var, guaranteed by semantic analysis"
                    );
                };
                let res = self.lower_expression(*exp2, instrs);
                instrs.push(Instruction::Copy {
                    src: res,
                    dest: Value::Var(v.clone()),
                });
                Value::Var(v)
            }
            parse::Expression::PrefixIncr(exp) => {
                let parse::Expression::Var(v) = *exp else {
                    unreachable!(
                        "Assignment lvalue must be a Var, guaranteed by semantic analysis"
                    );
                };
                instrs.push(Instruction::Binary {
                    binary_operator: BinaryOp::Add,
                    src1: Value::Var(v.clone()),
                    src2: Value::Constant(1),
                    dest: Value::Var(v.clone()),
                });
                Value::Var(v)
            }
            parse::Expression::PrefixDecr(exp) => {
                let parse::Expression::Var(v) = *exp else {
                    unreachable!(
                        "Assignment lvalue must be a Var, guaranteed by semantic analysis"
                    );
                };
                instrs.push(Instruction::Binary {
                    binary_operator: BinaryOp::Subtract,
                    src1: Value::Var(v.clone()),
                    src2: Value::Constant(1),
                    dest: Value::Var(v.clone()),
                });
                Value::Var(v)
            }
            parse::Expression::PostfixIncr(exp) => {
                let parse::Expression::Var(v) = *exp else {
                    unreachable!(
                        "Assignment lvalue must be a Var, guaranteed by semantic analysis"
                    );
                };
                let var = format!("tmp.{}", self.tmp_var_count);
                self.tmp_var_count += 1;
                instrs.push(Instruction::Copy {
                    src: Value::Var(v.clone()),
                    dest: Value::Var(var.clone()),
                });
                instrs.push(Instruction::Binary {
                    binary_operator: BinaryOp::Add,
                    src1: Value::Var(v.clone()),
                    src2: Value::Constant(1),
                    dest: Value::Var(v),
                });
                Value::Var(var)
            }
            parse::Expression::PostfixDecr(exp) => {
                let parse::Expression::Var(v) = *exp else {
                    unreachable!(
                        "Assignment lvalue must be a Var, guaranteed by semantic analysis"
                    );
                };
                let var = format!("tmp.{}", self.tmp_var_count);
                self.tmp_var_count += 1;
                instrs.push(Instruction::Copy {
                    src: Value::Var(v.clone()),
                    dest: Value::Var(var.clone()),
                });
                instrs.push(Instruction::Binary {
                    binary_operator: BinaryOp::Subtract,
                    src1: Value::Var(v.clone()),
                    src2: Value::Constant(1),
                    dest: Value::Var(v),
                });
                Value::Var(var)
            }
        }
    }
}

pub fn transform(program: parse::Program, var_count: &mut usize) -> Program {
    let mut transformer = TackyTransformer {
        tmp_var_count: *var_count,
        and_false_label: 0,
        or_true_label: 0,
    };
    transformer.lower_program(program)
}
