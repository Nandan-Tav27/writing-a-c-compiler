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
}

impl From<parse::UnaryOp> for UnaryOp {
    fn from(op: parse::UnaryOp) -> Self {
        match op {
            parse::UnaryOp::Complement => UnaryOp::Complement,
            parse::UnaryOp::Negation => UnaryOp::Negation,
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
    BitwiseAND,
    BitwiseXOR,
    BitwiseOR,
}

impl From<parse::BinaryOp> for BinaryOp {
    fn from(op: parse::BinaryOp) -> BinaryOp {
        match op {
            parse::BinaryOp::Add => BinaryOp::Add,
            parse::BinaryOp::Subtract => BinaryOp::Subtract,
            parse::BinaryOp::Multiply => BinaryOp::Multiply,
            parse::BinaryOp::Divide => BinaryOp::Divide,
            parse::BinaryOp::Remainder => BinaryOp::Remainder,
            parse::BinaryOp::LeftShift => BinaryOp::LeftShift,
            parse::BinaryOp::RightShift => BinaryOp::RightShift,
            parse::BinaryOp::BitwiseAND => BinaryOp::BitwiseAND,
            parse::BinaryOp::BitwiseXOR => BinaryOp::BitwiseXOR,
            parse::BinaryOp::BitwiseOR => BinaryOp::BitwiseOR,
        }
    }
}

pub struct TackyTransformer {
    var_count: usize,
}

impl TackyTransformer {
    fn lower_program(&mut self, program: parse::Program) -> Program {
        let function_def = self.lower_function_def(program.function_def);
        Program { function_def }
    }

    fn lower_function_def(&mut self, function_def: parse::FunctionDef) -> FunctionDef {
        let name = function_def.name;
        let body = self.lower_statement(function_def.body);
        FunctionDef { name, body }
    }

    fn lower_statement(&mut self, statement: parse::Statement) -> Vec<Instruction> {
        let mut instrs: Vec<Instruction> = Vec::new();
        match statement {
            parse::Statement::Return(exp) => {
                let val = self.lower_expression(exp, &mut instrs);
                instrs.push(Instruction::Ret(val));
            }
        }
        instrs
    }

    fn lower_expression(&mut self, exp: parse::Expression, instrs: &mut Vec<Instruction>) -> Value {
        match exp {
            parse::Expression::Constant(val) => Value::Constant(val),
            parse::Expression::Unary(op, exp) => {
                let unary_operator: UnaryOp = op.into();
                let src = self.lower_expression(*exp, instrs);
                let var = format!("tmp.{}", self.var_count);
                let dest = Value::Var(var.clone());
                self.var_count += 1;
                let instr = Instruction::Unary {
                    unary_operator,
                    src,
                    dest,
                };
                instrs.push(instr);
                Value::Var(var)
            }
            parse::Expression::Binary(op, exp1, exp2) => {
                let binary_operator: BinaryOp = op.into();
                let src1 = self.lower_expression(*exp1, instrs);
                let src2 = self.lower_expression(*exp2, instrs);
                let var = format!("tmp.{}", self.var_count);
                let dest = Value::Var(var.clone());
                self.var_count += 1;
                let instr = Instruction::Binary {
                    binary_operator,
                    src1,
                    src2,
                    dest,
                };
                instrs.push(instr);
                Value::Var(var)
            }
        }
    }
}

pub fn transform(program: parse::Program) -> Program {
    let mut transformer = TackyTransformer { var_count: 0 };
    transformer.lower_program(program)
}
