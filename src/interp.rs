use std::{collections::HashMap, ops::{Deref, Index}, fmt::Display};
use lala::parser::*;

use anyhow::anyhow;

#[derive(Debug, PartialEq, Clone)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Index<usize> for Matrix {
    type Output = [f64];

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index * self.cols..(index + 1) * self.cols]
    }
}

pub enum LalaType{
    Integer(i32),
    Matrix(Matrix),
    Double(f64)
}

impl Display for LalaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            LalaType::Integer(i) => write!(f, "{}", i.to_string())?,
            LalaType::Double(d) => write!(f, "{}", d.to_string())?,
            LalaType::Matrix(m) => {
                for r in 0..m.rows {
                    write!(f, "[")?;
                    for c in 0..m.cols {
                        if c == m.cols - 1 { write!(f, "{:.2}", m[r][c])?; } else { write!(f, "{:.2} ", m[r][c])?; }   
                    }
                    writeln!(f, "]")?;
                }
            }
        };
        Ok(())
    }
}

// later, return result<matrix, error>
fn construct_matrix(v: &Vec<Vec<AstNode>>) -> Matrix {
    let rows = v.len();
    let cols = v[0].len();
    let mut mat: Vec<f64> = vec![0.0; rows*cols];

    for row in 0..rows {
        for col in 0..cols {
            match &v[row][col] {
                AstNode::Integer(i) => mat[row*cols + col] = *i as f64,
                AstNode::DoublePrecisionFloat(d) => mat[row*cols + col] = *d,
                err => panic!("{:?} not allowed in matrix definition", err)
            }
        }
    }


    Matrix { rows, cols, data: mat }
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
        AstNode::Matrix(v) => {
            let data = construct_matrix(v);
            match env.insert(ident.to_string(), LalaType::Matrix(data)) {
                _ => Ok(())
            }
        }
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