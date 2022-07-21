use crate::allocator::Allocator;
use crate::storage::Storage;
use crate::{Const, DefaultAllocator, Dim, Matrix, OVector, RowOVector, Scalar, VectorSlice, U1};
use num::Zero;
use simba::scalar::{ClosedAdd, Field, SupersetOf};

/// # Folding on columns and rows
impl<T: Scalar, R: Dim, C: Dim, S: Storage<T, R, C>> Matrix<T, R, C, S> {
    /// Returns a row vector where each element is the result of the application of `f` on the
    /// corresponding column of the original matrix.
    #[inline]
    pub fn compress_rows(
        &self,
        f: impl Fn(VectorSlice<T, R, S::RStride, S::CStride>) -> T,
    ) -> RowOVector<T, C>
    where
        DefaultAllocator: Allocator<T, U1, C>,
    {
        let ncols = self.data.shape().1;
        let mut res: RowOVector<T, C> =
            unsafe { crate::unimplemented_or_uninitialized_generic!(Const::<1>, ncols) };

        for i in 0..ncols.value() {
            // TODO: avoid bound checking of column.
            unsafe {
                *res.get_unchecked_mut((0, i)) = f(self.column(i));
            }
        }

        res
    }

    /// Returns a column vector where each element is the result of the application of `f` on the
    /// corresponding column of the original matrix.
    ///
    /// This is the same as `self.compress_rows(f).transpose()`.
    #[inline]
    pub fn compress_rows_tr(
        &self,
        f: impl Fn(VectorSlice<T, R, S::RStride, S::CStride>) -> T,
    ) -> OVector<T, C>
    where
        DefaultAllocator: Allocator<T, C>,
    {
        let ncols = self.data.shape().1;
        let mut res: OVector<T, C> =
            unsafe { crate::unimplemented_or_uninitialized_generic!(ncols, Const::<1>) };

        for i in 0..ncols.value() {
            // TODO: avoid bound checking of column.
            unsafe {
                *res.vget_unchecked_mut(i) = f(self.column(i));
            }
        }

        res
    }

    /// Returns a column vector resulting from the folding of `f` on each column of this matrix.
    #[inline]
    pub fn compress_columns(
        &self,
        init: OVector<T, R>,
        f: impl Fn(&mut OVector<T, R>, VectorSlice<T, R, S::RStride, S::CStride>),
    ) -> OVector<T, R>
    where
        DefaultAllocator: Allocator<T, R>,
    {
        let mut res = init;

        for i in 0..self.ncols() {
            f(&mut res, self.column(i))
        }

        res
    }
}

/// # Common statistics operations
impl<T: Scalar, R: Dim, C: Dim, S: Storage<T, R, C>> Matrix<T, R, C, S> {
    /*
     *
     * Sum computation.
     *
     */
    /// The sum of all the elements of this matrix.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::Matrix2x3;
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.sum(), 21.0);
    /// ```
    #[inline]
    pub fn sum(&self) -> T
    where
        T: ClosedAdd + Zero,
    {
        self.iter().cloned().fold(T::zero(), |a, b| a + b)
    }

    /// The sum of all the rows of this matrix.
    ///
    /// Use `.row_variance_tr` if you need the result in a column vector instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Matrix2x3, Matrix3x2};
    /// # use nalgebra::{RowVector2, RowVector3};
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.row_sum(), RowVector3::new(5.0, 7.0, 9.0));
    ///
    /// let mint = Matrix3x2::new(1,2,3,4,5,6);
    /// assert_eq!(mint.row_sum(), RowVector2::new(9,12));
    /// ```
    #[inline]
    pub fn row_sum(&self) -> RowOVector<T, C>
    where
        T: ClosedAdd + Zero,
        DefaultAllocator: Allocator<T, U1, C>,
    {
        self.compress_rows(|col| col.sum())
    }

    /// The sum of all the rows of this matrix. The result is transposed and returned as a column vector.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Matrix2x3, Matrix3x2};
    /// # use nalgebra::{Vector2, Vector3};
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.row_sum_tr(), Vector3::new(5.0, 7.0, 9.0));
    ///
    /// let mint = Matrix3x2::new(1,2,3,4,5,6);
    /// assert_eq!(mint.row_sum_tr(), Vector2::new(9,12));
    /// ```
    #[inline]
    pub fn row_sum_tr(&self) -> OVector<T, C>
    where
        T: ClosedAdd + Zero,
        DefaultAllocator: Allocator<T, C>,
    {
        self.compress_rows_tr(|col| col.sum())
    }

    /// The sum of all the columns of this matrix.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Matrix2x3, Matrix3x2};
    /// # use nalgebra::{Vector2, Vector3};
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.column_sum(), Vector2::new(6.0, 15.0));
    ///
    /// let mint = Matrix3x2::new(1,2,3,4,5,6);
    /// assert_eq!(mint.column_sum(), Vector3::new(3,7,11));
    /// ```
    #[inline]
    pub fn column_sum(&self) -> OVector<T, R>
    where
        T: ClosedAdd + Zero,
        DefaultAllocator: Allocator<T, R>,
    {
        let nrows = self.data.shape().0;
        self.compress_columns(OVector::zeros_generic(nrows, Const::<1>), |out, col| {
            *out += col;
        })
    }

    /*
     *
     * Variance computation.
     *
     */
    /// The variance of all the elements of this matrix.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use] extern crate approx;
    /// # use nalgebra::Matrix2x3;
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_relative_eq!(m.variance(), 35.0 / 12.0, epsilon = 1.0e-8);
    /// ```
    #[inline]
    pub fn variance(&self) -> T
    where
        T: Field + SupersetOf<f64>,
    {
        if self.is_empty() {
            T::zero()
        } else {
            let val = self.iter().cloned().fold((T::zero(), T::zero()), |a, b| {
                (a.0 + b.inlined_clone() * b.inlined_clone(), a.1 + b)
            });
            let denom = T::one() / crate::convert::<_, T>(self.len() as f64);
            let vd = val.1 * denom.inlined_clone();
            val.0 * denom - vd.inlined_clone() * vd
        }
    }

    /// The variance of all the rows of this matrix.
    ///
    /// Use `.row_variance_tr` if you need the result in a column vector instead.
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Matrix2x3, RowVector3};
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.row_variance(), RowVector3::new(2.25, 2.25, 2.25));
    /// ```
    #[inline]
    pub fn row_variance(&self) -> RowOVector<T, C>
    where
        T: Field + SupersetOf<f64>,
        DefaultAllocator: Allocator<T, U1, C>,
    {
        self.compress_rows(|col| col.variance())
    }

    /// The variance of all the rows of this matrix. The result is transposed and returned as a column vector.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Matrix2x3, Vector3};
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.row_variance_tr(), Vector3::new(2.25, 2.25, 2.25));
    /// ```
    #[inline]
    pub fn row_variance_tr(&self) -> OVector<T, C>
    where
        T: Field + SupersetOf<f64>,
        DefaultAllocator: Allocator<T, C>,
    {
        self.compress_rows_tr(|col| col.variance())
    }

    /// The variance of all the columns of this matrix.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use] extern crate approx;
    /// # use nalgebra::{Matrix2x3, Vector2};
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_relative_eq!(m.column_variance(), Vector2::new(2.0 / 3.0, 2.0 / 3.0), epsilon = 1.0e-8);
    /// ```
    #[inline]
    pub fn column_variance(&self) -> OVector<T, R>
    where
        T: Field + SupersetOf<f64>,
        DefaultAllocator: Allocator<T, R>,
    {
        let (nrows, ncols) = self.data.shape();

        let mut mean = self.column_mean();
        mean.apply(|e| -(e.inlined_clone() * e));

        let denom = T::one() / crate::convert::<_, T>(ncols.value() as f64);
        self.compress_columns(mean, |out, col| {
            for i in 0..nrows.value() {
                unsafe {
                    let val = col.vget_unchecked(i);
                    *out.vget_unchecked_mut(i) +=
                        denom.inlined_clone() * val.inlined_clone() * val.inlined_clone()
                }
            }
        })
    }

    /*
     *
     * Mean computation.
     *
     */
    /// The mean of all the elements of this matrix.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::Matrix2x3;
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.mean(), 3.5);
    /// ```
    #[inline]
    pub fn mean(&self) -> T
    where
        T: Field + SupersetOf<f64>,
    {
        if self.is_empty() {
            T::zero()
        } else {
            self.sum() / crate::convert(self.len() as f64)
        }
    }

    /// The mean of all the rows of this matrix.
    ///
    /// Use `.row_mean_tr` if you need the result in a column vector instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Matrix2x3, RowVector3};
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.row_mean(), RowVector3::new(2.5, 3.5, 4.5));
    /// ```
    #[inline]
    pub fn row_mean(&self) -> RowOVector<T, C>
    where
        T: Field + SupersetOf<f64>,
        DefaultAllocator: Allocator<T, U1, C>,
    {
        self.compress_rows(|col| col.mean())
    }

    /// The mean of all the rows of this matrix. The result is transposed and returned as a column vector.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Matrix2x3, Vector3};
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.row_mean_tr(), Vector3::new(2.5, 3.5, 4.5));
    /// ```
    #[inline]
    pub fn row_mean_tr(&self) -> OVector<T, C>
    where
        T: Field + SupersetOf<f64>,
        DefaultAllocator: Allocator<T, C>,
    {
        self.compress_rows_tr(|col| col.mean())
    }

    /// The mean of all the columns of this matrix.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Matrix2x3, Vector2};
    ///
    /// let m = Matrix2x3::new(1.0, 2.0, 3.0,
    ///                        4.0, 5.0, 6.0);
    /// assert_eq!(m.column_mean(), Vector2::new(2.0, 5.0));
    /// ```
    #[inline]
    pub fn column_mean(&self) -> OVector<T, R>
    where
        T: Field + SupersetOf<f64>,
        DefaultAllocator: Allocator<T, R>,
    {
        let (nrows, ncols) = self.data.shape();
        let denom = T::one() / crate::convert::<_, T>(ncols.value() as f64);
        self.compress_columns(OVector::zeros_generic(nrows, Const::<1>), |out, col| {
            out.axpy(denom.inlined_clone(), &col, T::one())
        })
    }
}
