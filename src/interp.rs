use anyhow::anyhow;
use super::parser::*;
use super::types::*;
use super::commands;
use std::{collections::HashMap, ops::Deref};

#[inline]
fn get_value<'a>(map: &'a HashMap<String, LalaType>, key: &'a String) -> LalaType {
    match map.get(key) {
        Some(val) => val.clone(),
        None => panic!("Key not found in the hashmap: {}", key),
    }
}

#[inline]
fn eval_expr(env: &mut HashMap<String, LalaType>, expr: &Box<AstNode>, func: &str) -> LalaType {
    match expr.deref() {
        AstNode::Ident(id) => get_value(env, id),
        AstNode::MonadicOp { verb, expr } => eval_monadic_op(expr, env, verb),
        AstNode::DyadicOp { verb, lhs, rhs } => eval_dyadic_op(lhs, rhs, env, verb),
        AstNode::Matrix(m) => LalaType::Matrix(construct_matrix(m)),
        _ => panic!("Can only call {} on a matrix.", func),
    }
}

fn eval_monadic_op(
    expr: &Box<AstNode>,
    env: &mut HashMap<String, LalaType>,
    verb: &MonadicVerb,
) -> LalaType {
    let func = verb.to_string();
    let matrix = match eval_expr(env, expr, &func) {
        LalaType::Matrix(mat) => mat,
        _ => panic!("Can only call {func} on a matrix."),
    };
    match verb {
        MonadicVerb::Inverse => LalaType::Matrix(matrix.inverse()),
        MonadicVerb::Rank => LalaType::Integer(matrix.rank()),
        MonadicVerb::RREF => LalaType::Matrix(matrix.rref()),
        MonadicVerb::Transpose => LalaType::Matrix(matrix.transpose()),
        MonadicVerb::Determinant => LalaType::Double(matrix.det()),
    }
}

fn eval_dyadic_op(
    lhs: &Box<AstNode>,
    rhs: &Box<AstNode>,
    env: &mut HashMap<String, LalaType>,
    verb: &DyadicVerb,
) -> LalaType {
    let func = verb.to_string();
    let leftside = if let LalaType::Matrix(left) = eval_expr(env, lhs, &func) {
        left
    } else {
        panic!("can only call {func} on a matrix");
    };
    let rightside = if let LalaType::Matrix(right) = eval_expr(env, rhs, &func) {
        right
    } else {
        panic!("can only call {func} on a matrix");
    };
    match verb {
        DyadicVerb::Dot => LalaType::Matrix(leftside.dot(rightside.clone())),
        DyadicVerb::Plus => LalaType::Matrix(leftside.combine(rightside, |a, b| a + b)),
        DyadicVerb::Times => LalaType::Matrix(leftside.combine(rightside, |a, b| a * b)),
    }
}

fn eval_assignment(
    ident: &String,
    expr: &Box<AstNode>,
    env: &mut HashMap<String, LalaType>,
) -> Result<(), anyhow::Error> {
    match expr.deref() {
        AstNode::Integer(scalar) => match env.insert(ident.to_string(), LalaType::Integer(*scalar))
        {
            _ => Ok(()),
        },
        AstNode::DoublePrecisionFloat(scalar) => {
            match env.insert(ident.to_string(), LalaType::Double(*scalar)) {
                _ => Ok(()),
            }
        }
        AstNode::Matrix(v) => {
            let mat = construct_matrix(v);
            match env.insert(ident.to_string(), LalaType::Matrix(mat)) {
                _ => Ok(()),
            }
        }
        AstNode::MonadicOp { verb, expr } => {
            let result = eval_monadic_op(expr, env, verb);
            match env.insert(ident.to_string(), result) {
                _ => return Ok(()),
            }
        }
        AstNode::DyadicOp { verb, lhs, rhs } => {
            let result = eval_dyadic_op(lhs, rhs, env, verb);
            match env.insert(ident.to_string(), result) {
                _ => return Ok(()),
            }
        }
        _ => Err(anyhow!("bruh")),
    }
}

fn eval_cmd(cmd: &str, cmd_params: &Vec<&str>, env: &mut HashMap<String, LalaType>) -> Result<(), anyhow::Error> {
    match cmd {
        "link" => commands::link(cmd_params, env),
        "interp" |
        "dbg" |
        _ => todo!()
    }
}

pub fn interp(
    ast: &Vec<Box<AstNode>>,
    map: Option<&mut HashMap<String, LalaType>>,
    linking: bool
) -> Result<(), anyhow::Error> {
    let mut binding = HashMap::new();
    #[allow(unused_mut)]
    let mut env: &mut HashMap<String, LalaType> = match map {
        Some(m) => m,
        None => &mut binding,
    };
    for node in ast {
        let _ = match node.deref() {
            AstNode::Assignment { ident, expr } => eval_assignment(ident, expr, env),
            AstNode::MonadicOp { verb, expr } => {
                let result = eval_monadic_op(expr, env, verb);
                if !linking { println!("{result}"); }
                Ok(())
            }
            AstNode::DyadicOp { verb, lhs, rhs } => {
                let result = eval_dyadic_op(lhs, rhs, env, verb);
                if !linking { println!("{result}"); }
                Ok(())
            }
            AstNode::Ident(var) => {
                if !linking { println!("{}", env.get(var).unwrap()); }
                Ok(())
            }
            AstNode::Command((cmd, cmd_params)) => eval_cmd(*cmd, cmd_params, env),
            bad_line => panic!("Invalid line: {:?}", bad_line),
        };
    }

    Ok(())
}
