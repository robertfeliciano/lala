#![allow(unused)]
use anyhow::anyhow;
use lala::matrix::*;
use lala::parser::*;
use lala::types::*;
use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, Index, IndexMut},
};

fn eval_monadic_op(
    ident: &String,
    expr: &Box<AstNode>,
    env: &mut HashMap<String, LalaType>,
    verb: &MonadicVerb,
) -> Result<(), anyhow::Error> {
    match verb {
        MonadicVerb::Inverse => {
            let m = match expr.deref() {
                AstNode::Ident(id) => match env.get(id).unwrap() {
                    LalaType::Matrix(m) => m,
                    _ => panic!("not figured out yet"),
                },
                _ => panic!("also not figured out yet"),
            };
            match env.insert(ident.to_string(), LalaType::Matrix(m.inverse())) {
                _ => return Ok(()),
            }
        }
        MonadicVerb::Rank => {
            todo!()
        }
        MonadicVerb::RREF => {
            let m = match expr.deref() {
                AstNode::Ident(id) => match env.get(id).unwrap() {
                    LalaType::Matrix(m) => m,
                    _ => panic!("not figured out yet"),
                },
                _ => panic!("also not figured out yet"),
            };
            match env.insert(ident.to_string(), LalaType::Matrix(m.rref())) {
                _ => return Ok(()),
            }
        }
        MonadicVerb::Transpose => {
            let m = match expr.deref() {
                AstNode::Ident(id) => match env.get(id).unwrap() {
                    LalaType::Matrix(m) => m,
                    _ => panic!("not figured out yet"),
                },
                _ => panic!("also not figured out yet"),
            };
            match env.insert(ident.to_string(), LalaType::Matrix(m.transpose())) {
                _ => return Ok(()),
            }
        }
        MonadicVerb::Determinant => {
            let m = match expr.deref() {
                AstNode::Ident(id) => match env.get(id).unwrap() {
                    LalaType::Matrix(m) => m,
                    _ => panic!("not figured out yet"),
                },
                _ => panic!("also not figured out yet"),

            };
            match env.insert(ident.to_string(), LalaType::Double(m.det())) {
                _ => return Ok(()),
            }
        }
    };
}

fn eval_dyadic_op(
    ident: &String,
    lhs: &Box<AstNode>,
    rhs: &Box<AstNode>,
    env: &mut HashMap<String, LalaType>,
    verb: &DyadicVerb,
) -> Result<(), anyhow::Error> {
    match verb {
        DyadicVerb::Dot => {
            let leftside = match lhs.deref() {
                AstNode::Ident(id) => match env.get(id).unwrap() {
                    LalaType::Matrix(m) => m,
                    _ => panic!("not allowed"),
                },
                // AstNode::Matrix(m) => &LalaType::Matrix(construct_matrix(&m)),
                _ => panic!("oops"),
            };
            let rightside = match rhs.deref() {
                AstNode::Ident(id) => match env.get(id).unwrap() {
                    LalaType::Matrix(m) => m,
                    _ => panic!("not allowed"),
                },
                // AstNode::Matrix(m) => &LalaType::Matrix(construct_matrix(&m)),
                _ => panic!("oops"),
            };
            match env.insert(
                ident.to_string(),
                LalaType::Matrix(leftside.dot(rightside.clone())),
            ) {
                _ => Ok(()),
            }

            // Ok(())
        }
        DyadicVerb::Plus => todo!(),
        DyadicVerb::Times => todo!(),
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
            let data = lala::types::construct_matrix(v);
            match env.insert(ident.to_string(), LalaType::Matrix(data)) {
                _ => Ok(()),
            }
        }
        AstNode::MonadicOp { verb, expr } => 
            eval_monadic_op(ident, expr, env, verb),
        AstNode::DyadicOp { verb, lhs, rhs } => 
            eval_dyadic_op(ident, lhs, rhs, env, verb),
        _ => Err(anyhow!("bruh")),
    }
}

pub fn interp(ast: &Vec<Box<AstNode>>) -> Result<(), anyhow::Error> {
    let mut env: HashMap<String, LalaType> = HashMap::new();
    for node in ast {
        let _ = match node.deref() {
            AstNode::Assignment { ident, expr } => eval_assignment(ident, expr, &mut env),
            AstNode::MonadicOp { verb, expr } => todo!(),
            // move dyadic logic in eval_assignment into eval_dyadicop to reuse here
            AstNode::DyadicOp { verb, lhs, rhs } => todo!(),
            AstNode::Ident(var) => {
                println!("{}", env.get(var).unwrap());
                Ok(())
            }
            bad_line => panic!("Invalid line: {:?}", bad_line),
        };
    }

    Ok(())
}
