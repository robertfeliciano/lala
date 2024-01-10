use anyhow::anyhow;
use lala::parser::*;
use lala::types::*;
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
    let result = match verb {
        MonadicVerb::Inverse => {
            let m = eval_expr(env, expr, "inverse");
            let matrix = match m {
                LalaType::Matrix(mat) => mat,
                _ => panic!("Can only call inverse on a matrix."),
            };
            LalaType::Matrix(matrix.inverse())
        }
        MonadicVerb::Rank => {
            let m = eval_expr(env, expr, "rank");
            let matrix = match m {
                LalaType::Matrix(mat) => mat,
                _ => panic!("Can only call rank on a matrix."),
            };
            LalaType::Integer(matrix.rank())
        }
        MonadicVerb::RREF => {
            let m = eval_expr(env, expr, "rref");
            let matrix = match m {
                LalaType::Matrix(mat) => mat,
                _ => panic!("Can only call rref on a matrix."),
            };
            LalaType::Matrix(matrix.rref())
        }
        MonadicVerb::Transpose => {
            let m = eval_expr(env, expr, "transpose");
            let matrix = match m {
                LalaType::Matrix(mat) => mat,
                _ => panic!("Can only call transpose on a matrix."),
            };
            LalaType::Matrix(matrix.transpose())
        }
        MonadicVerb::Determinant => {
            let m = eval_expr(env, expr, "determinant");
            let matrix = match m {
                LalaType::Matrix(mat) => mat,
                _ => panic!("Can only call determinant on a matrix."),
            };
            LalaType::Double(matrix.det())
        }
    };
    return result;
}

fn eval_dyadic_op(
    lhs: &Box<AstNode>,
    rhs: &Box<AstNode>,
    env: &mut HashMap<String, LalaType>,
    verb: &DyadicVerb,
) -> LalaType {
    match verb {
        DyadicVerb::Dot => {
            let leftside = match lhs.deref() {
                AstNode::Ident(id) => match env.get(id).unwrap() {
                    LalaType::Matrix(m) => m,
                    _ => panic!("not allowed"),
                },
                // AstNode::Matrix(m) => {
                //     let x = &construct_matrix(m);
                //     x
                // }
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
            let result = LalaType::Matrix(leftside.dot(rightside.clone()));
            return result;

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
            let mat = lala::types::construct_matrix(v);
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

pub fn interp(
    ast: &Vec<Box<AstNode>>,
    map: Option<&mut HashMap<String, LalaType>>,
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
                println!("{result}");
                Ok(())
            }
            AstNode::DyadicOp { verb, lhs, rhs } => {
                let result = eval_dyadic_op(lhs, rhs, env, verb);
                println!("{result}");
                Ok(())
            }
            AstNode::Ident(var) => {
                println!("{}", env.get(var).unwrap());
                Ok(())
            }
            bad_line => panic!("Invalid line: {:?}", bad_line),
        };
    }

    Ok(())
}
