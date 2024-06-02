use super::linalg::Matrix;
use super::parser::{AstNode, DyadicVerb, MonadicVerb};
use anyhow::{anyhow, Error};
use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum LalaType<'a> {
    Integer(i32),
    Double(f64),
    Matrix(Matrix),
    Fun((String, Vec<AstNode<'a>>, Vec<AstNode<'a>>)),
}

impl Display for LalaType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LalaType::Integer(i) => write!(f, "{}", i.to_string())?,
            LalaType::Double(d) => write!(f, "{}", d.to_string())?,
            LalaType::Matrix(m) => {
                for r in 0..m.rows {
                    write!(f, "[")?;
                    for c in 0..m.cols {
                        if c == m.cols - 1 {
                            write!(f, "{:.2}", m[r][c])?;
                        } else {
                            write!(f, "{:.2} ", m[r][c])?;
                        }
                    }
                    writeln!(f, "]")?;
                }
            }
            LalaType::Fun((name, param_list, _body)) => {
                writeln!(f, "FUN {name}")?;
                writeln!(
                    f,
                    "params: [{:?}]",
                    param_list
                        .iter()
                        .map(|node| {
                            if let AstNode::Ident(id) = node {
                                format!("{id} ")
                            } else {
                                "_".to_string() // Placeholder for other AstNode variants
                            }
                        })
                        .collect::<Vec<String>>()
                )?;
            }
        };
        Ok(())
    }
}

impl ToString for MonadicVerb {
    fn to_string(&self) -> String {
        match self {
            MonadicVerb::Rank => String::from("matrix rank"),
            MonadicVerb::Inverse => String::from("matrix inverse"),
            MonadicVerb::RREF => String::from("matrix rref"),
            MonadicVerb::Transpose => String::from("matrix transpose"),
            MonadicVerb::Determinant => String::from("matrix determinant"),
        }
    }
}

impl ToString for DyadicVerb {
    fn to_string(&self) -> String {
        match self {
            DyadicVerb::Dot => String::from("dot product"),
            DyadicVerb::Plus => String::from("matrix addition"),
            DyadicVerb::Times => String::from("matrix multiplication"),
        }
    }
}

// later, return result<matrix, error>
pub fn construct_matrix(v: &Vec<Vec<AstNode>>) -> Result<Matrix, Error> {
    let rows = v.len();
    let cols = v[0].len();
    let mut mat: Vec<f64> = vec![0.0; rows * cols];

    for row in 0..rows {
        for col in 0..cols {
            match &v[row][col] {
                AstNode::Integer(i) => mat[row * cols + col] = *i as f64,
                AstNode::DoublePrecisionFloat(d) => mat[row * cols + col] = *d,
                err => return Err(anyhow!("{:?} not allowed in matrix definition", err)),
            }
        }
    }
    Ok(Matrix {
        rows,
        cols,
        data: mat,
    })
}
