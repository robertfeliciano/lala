use std::ops::{Index, IndexMut};

#[derive(Debug, PartialEq, Clone)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn dot(&self, b: Self) -> Self {
        if self.rows != b.cols || self.cols != b.rows {
            panic!(
                "Dimensions not matched. M1 is {} by {}, M2 is {} by {}.",
                self.rows, self.cols, b.rows, b.cols
            );
        }
        let mut dp = Self::new(self.rows, b.cols);
        for i in 0..self.rows {
            for j in 0..b.cols {
                let mut sum = 0.0;
                for k in 0..b.rows {
                    sum += self[i][k] * b[k][j];
                }
                dp[i][j] = sum;
            }
        }
        dp
    }

    pub fn rank(&self) -> i32 {
        let reduced = self.rref().data;
        let rows = self.rows;
        let cols = self.cols;
        let mut rank = 0;
        for r in 0..rows {
            let row = &reduced[r * cols..r * cols + cols];
            let (prefix, aligned, suffix) = unsafe { row.align_to::<f64>() };
            if !(prefix.iter().all(|&x| x == 0f64)
                && suffix.iter().all(|&x| x == 0f64)
                && aligned.iter().all(|&x| x == 0f64))
            {
                rank += 1;
            }
        }
        rank
    }

    pub fn rref(&self) -> Self {
        let mut reduced = self.clone();
        if reduced[0][0] == 0.0 {
            reduced.swap_rows(0);
        }
        let mut lead: usize = 0;
        let rows = reduced.rows;
        while lead < rows {
            for r in 0..rows {
                let div = reduced[lead][lead];
                let mult = reduced[r][lead] / div;

                if r == lead {
                    reduced[lead]
                        .iter_mut()
                        .for_each(|elem| *elem = (*elem) / div);
                } else {
                    for c in 0..reduced.cols {
                        reduced[r][c] -= reduced[lead][c] * mult;
                    }
                }
            }
            lead += 1;
        }
        reduced.correct();
        reduced
    }

    pub fn cofactor(&self, expanded_row: usize, j: usize) -> f64 {
        let mut cut: Vec<Vec<f64>> = Vec::new();
        for r in 0..self.rows {
            if r == expanded_row {
                continue;
            }
            let mut v: Vec<f64> = Vec::new();
            for c in 0..self.cols {
                if c == j {
                    continue;
                }
                v.push(self[r][c]);
            }
            cut.push(v);
        }
        let flattened = cut.clone().into_iter().flatten().collect();
        let n_r = cut.len();
        let n_c = cut[0].len();
        let minor = Self {
            rows: n_r,
            cols: n_c,
            data: flattened,
        }
        .det();
        let base: i32 = -1;
        minor * f64::from(base.pow((expanded_row + j) as u32))
    }

    pub fn det(&self) -> f64 {
        if self.rows != self.cols {
            panic!(
                "Determinant requires matrix to be a square. Input matrix was {:?}.",
                self
            );
        }
        if self.rows == 2 && self.cols == 2 {
            self[0][0] * self[1][1] - self[0][1] * self[1][0]
        } else {
            let row: usize = 1;
            let mut det = 0.0;

            for j in 0..self[row].len() {
                det += self.cofactor(row, j) * self[row][j];
            }
            det
        }
    }

    pub fn transpose(&self) -> Self {
        let mut t = Self::new(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                t[j][i] = self[i][j];
            }
        }
        t
    }

    #[allow(dead_code)]
    pub fn trace(&self) -> f64 {
        if self.rows != self.cols {
            panic!(
                "Trace requires matrix to be square. Input matrix was {:?}.",
                self
            );
        }
        let mut t: f64 = 0.0;
        for i in 0..self.rows {
            t += self[i][i];
        }
        t
    }

    pub fn inverse(&self) -> Self {
        let d = self.det();
        if d == 0.0 {
            panic!("Determinant is 0. No inverse.");
        }

        let mut inv = Self::new(self.rows, self.cols);

        for row in 0..self.rows {
            for col in 0..self.cols {
                inv[row][col] = self.cofactor(row, col);
            }
        }

        inv.correct();
        inv = inv.transpose();
        inv.apply(|x| x / d);
        inv
    }

    #[allow(dead_code)]
    pub fn identity(&mut self) {
        if self.rows != self.cols {
            panic!("Not a square matrix.");
        }
        for r in 0..self.rows {
            self[r][r] = 1.0;
        }
    }

    pub fn apply(&mut self, f: impl Fn(f64) -> f64) {
        self.data = self.data.iter().map(|elem| f(*elem)).collect()
    }

    pub fn combine(&self, b: Self, f: impl Fn(f64, f64) -> f64) -> Self {
        if self.rows != b.rows || self.cols != b.cols {
            panic!("Matrices must be of the same size.");
        }
        let mut new_matrix = Self::new(self.rows, self.cols);
        new_matrix.data = self
            .data
            .iter()
            .zip(b.data.iter())
            .map(|(a, b)| f(*a, *b))
            .collect();
        new_matrix
    }

    fn swap_rows(&mut self, row: usize) {
        let mut n_r = 0;
        for r in 0..self.rows {
            if self[r][0] > 0.0 {
                n_r = r;
                break;
            }
        }
        let temp: Vec<f64> = self[row].to_vec();
        for c in 0..self.cols {
            self[row][c] = self[n_r][c];
            self[n_r][c] = temp[n_r * self.cols + c];
        }
    }

    fn correct(&mut self) {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let elem = self[row][col];
                if elem == -0.0 {
                    self[row][col] = 0.0;
                }
                let floored = elem.floor();
                if elem - floored > 0.9999999 {
                    self[row][col] = elem.round();
                }
                if elem > 0.0 && elem < 0.000001 {
                    self[row][col] = 0.0;
                }
                if elem < 0.0 && elem > -0.00001 {
                    self[row][col] = 0.0;
                }
            }
        }
    }
}

impl Index<usize> for Matrix {
    type Output = [f64];

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index * self.cols..(index + 1) * self.cols]
    }
}

impl IndexMut<usize> for Matrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index * self.cols..(index + 1) * self.cols]
    }
}
