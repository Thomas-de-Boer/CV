//! A minimal dense matrix: `Vec<f32>`, row-major.

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f32>,
}

impl Matrix {
    /// A `rows × cols` matrix filled with zeros.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Matrix { rows, cols, data: vec![0.0; rows * cols] }
    }

    /// Wraps row-major data. Panics if the length is not `rows * cols`.
    pub fn from_vec(rows: usize, cols: usize, data: Vec<f32>) -> Self {
        assert_eq!(data.len(), rows * cols, "matrix data has the wrong length");
        Matrix { rows, cols, data }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.data[row * self.cols + col]
    }

    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        self.data[row * self.cols + col] = value;
    }

    pub fn row(&self, row: usize) -> &[f32] {
        &self.data[row * self.cols..(row + 1) * self.cols]
    }

    pub fn row_mut(&mut self, row: usize) -> &mut [f32] {
        &mut self.data[row * self.cols..(row + 1) * self.cols]
    }

    /// All values, row-major.
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Matrix × column vector. Panics if `v.len() != cols`.
    pub fn mul_vec(&self, v: &[f32]) -> Vec<f32> {
        assert_eq!(v.len(), self.cols, "vector length must equal matrix columns");
        (0..self.rows)
            .map(|r| self.row(r).iter().zip(v).map(|(w, x)| w * x).sum())
            .collect()
    }

    pub fn transpose(&self) -> Matrix {
        let mut t = Matrix::zeros(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                t.set(c, r, self.get(r, c));
            }
        }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_vec_matches_hand_computation() {
        // [1 2 3]   [1]   [ 1 + 0 - 3]   [-2]
        // [4 5 6] × [0] = [ 4 + 0 - 6] = [-2]
        //           [-1]
        let m = Matrix::from_vec(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(m.mul_vec(&[1.0, 0.0, -1.0]), vec![-2.0, -2.0]);
    }

    #[test]
    fn get_set_and_rows_are_row_major() {
        let mut m = Matrix::zeros(2, 2);
        m.set(0, 1, 7.0);
        m.set(1, 0, 3.0);
        assert_eq!(m.as_slice(), &[0.0, 7.0, 3.0, 0.0]);
        assert_eq!(m.row(1), &[3.0, 0.0]);
        assert_eq!(m.get(0, 1), 7.0);
    }

    #[test]
    fn transpose_swaps_rows_and_columns() {
        let m = Matrix::from_vec(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let t = m.transpose();
        assert_eq!((t.rows(), t.cols()), (3, 2));
        assert_eq!(t.as_slice(), &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }

    #[test]
    #[should_panic]
    fn mul_vec_rejects_wrong_length() {
        Matrix::zeros(2, 3).mul_vec(&[1.0, 2.0]);
    }
}
