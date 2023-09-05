use std::{collections::HashMap, ops::Deref, fmt::Display};
use lala::parser::*;
use anyhow::{anyhow, Context};

pub enum LalaType{
    Integer(i32),
    Matrix(Vec<f64>),
    Double(f64)
}

impl Display for LalaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            LalaType::Integer(i) => write!(f, "{}", i.to_string())?,
            LalaType::Double(d) => write!(f, "{}", d.to_string())?,
            LalaType::Matrix(m) => todo!(),
        };

        Ok(())
    }
}

fn eval_assignment(ident: &String, expr: &Box<AstNode>, env: &mut HashMap<String, LalaType>) -> Result<(), anyhow::Error>{
    match expr.deref() {
        AstNode::Integer(scalar) => 
            match env.insert(ident.to_string(), LalaType::Integer(*scalar)) {
                _ => Ok(()),
            },
        AstNode::DoublePrecisionFloat(scalar) => 
            match env.insert(ident.to_string(), LalaType::Double(*scalar)) {
                _ => Ok(()),
            },
        AstNode::Matrix(v) => todo!(),
        AstNode::MonadicOp { verb, expr } => todo!(),
        AstNode::DyadicOp { verb, lhs, rhs } => todo!(),
        _ => Err(anyhow!("bruh"))
    }
}

pub fn interp(ast: &Vec<Box<AstNode>>) -> Result<(), anyhow::Error>{
    let mut env: HashMap<String, LalaType> = HashMap::new();
    for node in ast {
        let _ = match node.deref() {
            AstNode::Assignment { ident, expr } => eval_assignment(ident, expr, &mut env),
            AstNode::MonadicOp { verb, expr } => todo!(),
            AstNode::DyadicOp { verb, lhs, rhs } => todo!(),
            AstNode::Ident(var) => {
                println!("{}", env.get(var).unwrap());
                Ok(())
            },
            bad_line => panic!("Invalid line: {:?}", bad_line),
        };
    }

    Ok(())
}