use crate::matrix::Matrix;
use crate::parser::AstNode;
use std::fmt::Display;

#[derive(Clone)]
pub enum LalaType {
    Integer(i32),
    Double(f64),
    Matrix(Matrix),
}

impl Display for LalaType {
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
        };
        Ok(())
    }
}

// later, return result<matrix, error>
pub fn construct_matrix(v: &Vec<Vec<AstNode>>) -> Matrix {
    let rows = v.len();
    let cols = v[0].len();
    let mut mat: Vec<f64> = vec![0.0; rows * cols];

    for row in 0..rows {
        for col in 0..cols {
            match &v[row][col] {
                AstNode::Integer(i) => mat[row * cols + col] = *i as f64,
                AstNode::DoublePrecisionFloat(d) => mat[row * cols + col] = *d,
                err => panic!("{:?} not allowed in matrix definition", err),
            }
        }
    }
    Matrix {
        rows,
        cols,
        data: mat,
    }
}
