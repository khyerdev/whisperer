#![allow(unused)]

pub type Matrix<T> = Vec<Vec<T>>;

#[macro_export]
macro_rules! square_matrix {
    ($t:ty, $sidelength:expr) => {
        {
            let mut rows: Matrix<$t> = Vec::with_capacity($sidelength);
            for _ in 0..$sidelength {
                rows.push(Vec::with_capacity($sidelength))
            }
            rows
        }
    };
}