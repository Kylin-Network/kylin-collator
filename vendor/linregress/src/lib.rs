/*!
  `linregress` provides an easy to use implementation of ordinary
  least squared linear regression with some basic statistics.
  See [`RegressionModel`] for details.

  The builder [`FormulaRegressionBuilder`] is used to construct a model from a
  table of data and an R-style formula or a list of columns to use.
  Currently only very simple formulae are supported,
  see [`FormulaRegressionBuilder.formula`] for details.

  # Example

  ```
  use linregress::{FormulaRegressionBuilder, RegressionDataBuilder};

  # use linregress::Error;
  # fn main() -> Result<(), Error> {
  let y = vec![1., 2. ,3. , 4., 5.];
  let x1 = vec![5., 4., 3., 2., 1.];
  let x2 = vec![729.53, 439.0367, 42.054, 1., 0.];
  let x3 = vec![258.589, 616.297, 215.061, 498.361, 0.];
  let data = vec![("Y", y), ("X1", x1), ("X2", x2), ("X3", x3)];
  let data = RegressionDataBuilder::new().build_from(data)?;
  let formula = "Y ~ X1 + X2 + X3";
  let model = FormulaRegressionBuilder::new()
      .data(&data)
      .formula(formula)
      .fit()?;
  let parameters = model.parameters;
  let standard_errors = model.se;
  let pvalues = model.pvalues;
  assert_eq!(
      parameters.pairs(),
      vec![
          ("X1", -0.9999999999999745),
          ("X2", 0.00000000000000005637851296924623),
          ("X3", 0.00000000000000008283304597789254),
      ]
  );
  assert_eq!(
      standard_errors.pairs(),
      vec![
          ("X1", 0.00000000000019226371555402852),
          ("X2", 0.0000000000000008718958950659518),
          ("X3", 0.0000000000000005323837152041135),
      ]
  );
  assert_eq!(
      pvalues.pairs(),
      vec![
          ("X1", 0.00000000000012239888283055414),
          ("X2", 0.9588921357097694),
          ("X3", 0.9017368322742073),
      ]
  );
  # Ok(())
  # }
  ```

  [`RegressionModel`]: struct.RegressionModel.html
  [`FormulaRegressionBuilder`]: struct.FormulaRegressionBuilder.html
  [`FormulaRegressionBuilder.formula`]: struct.FormulaRegressionBuilder.html#method.formula
*/

#![warn(rust_2018_idioms)]
#![cfg_attr(feature = "unstable", feature(test))]
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::iter;

use nalgebra::{DMatrix, DVector, RowDVector};

pub use error::{Error, InconsistentSlopes};
use special_functions::stdtr;

mod error;
mod special_functions;

macro_rules! ensure {
    ($predicate:expr, $error:expr) => {
        if !$predicate {
            return Err($error);
        }
    };
}

/// A builder to create and fit a linear regression model.
///
/// Given a dataset and a set of columns to use this builder
/// will produce an ordinary least squared linear regression model.
///
/// See [`formula`] and [`data`] for details on how to configure this builder.
///
/// The pseudo inverse method is used to fit the model.
///
/// # Usage
///
/// ```
/// use linregress::{FormulaRegressionBuilder, RegressionDataBuilder};
///
/// # use linregress::Error;
/// # fn main() -> Result<(), Error> {
/// let y = vec![1., 2. ,3., 4.];
/// let x = vec![4., 3., 2., 1.];
/// let data = vec![("Y", y), ("X", x)];
/// let data = RegressionDataBuilder::new().build_from(data)?;
/// let model = FormulaRegressionBuilder::new().data(&data).formula("Y ~ X").fit()?;
/// // Alternatively
/// let model = FormulaRegressionBuilder::new().data(&data).data_columns("Y", ["X"]).fit()?;
/// assert_eq!(model.parameters.intercept_value, 4.999999999999998);
/// assert_eq!(model.parameters.regressor_values[0], -0.9999999999999989);
/// assert_eq!(model.parameters.regressor_names[0], "X");
/// # Ok(())
/// # }
/// ```
///
/// [`formula`]: struct.FormulaRegressionBuilder.html#method.formula
/// [`data`]: struct.FormulaRegressionBuilder.html#method.data
#[derive(Debug, Clone)]
pub struct FormulaRegressionBuilder<'a> {
    data: Option<&'a RegressionData<'a>>,
    formula: Option<Cow<'a, str>>,
    columns: Option<(Cow<'a, str>, Vec<Cow<'a, str>>)>,
}

impl<'a> Default for FormulaRegressionBuilder<'a> {
    fn default() -> Self {
        FormulaRegressionBuilder::new()
    }
}

impl<'a> FormulaRegressionBuilder<'a> {
    /// Create as new FormulaRegressionBuilder with no data or formula set.
    pub fn new() -> Self {
        FormulaRegressionBuilder {
            data: None,
            formula: None,
            columns: None,
        }
    }

    /// Set the data to be used for the regression.
    ///
    /// The data has to be given as a reference to a [`RegressionData`] struct.
    /// See [`RegressionDataBuilder`] for details.
    ///
    /// [`RegressionData`]: struct.RegressionData.html
    /// [`RegressionDataBuilder`]: struct.RegressionDataBuilder.html
    pub fn data(mut self, data: &'a RegressionData<'a>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the formula to use for the regression.
    ///
    /// The expected format is `<regressand> ~ <regressor 1> + <regressor 2>`.
    ///
    /// E.g. for a regressand named Y and three regressors named A, B and C
    /// the correct format would be `Y ~ A + B + C`.
    ///
    /// Note that there is currently no special support for categorical variables.
    /// So if you have a categorical variable with more than two distinct values
    /// or values that are not `0` and `1` you will need to perform "dummy coding" yourself.
    ///
    /// Alternatively you can use [`data_columns`][Self::data_columns].
    pub fn formula<T: Into<Cow<'a, str>>>(mut self, formula: T) -> Self {
        self.formula = Some(formula.into());
        self
    }

    /// Set the columns to be used as regressand and regressors for the regression.
    ///
    /// Note that there is currently no special support for categorical variables.
    /// So if you have a categorical variable with more than two distinct values
    /// or values that are not `0` and `1` you will need to perform "dummy coding" yourself.
    ///
    /// Alternatively you can use [`formula`][Self::formula].
    pub fn data_columns<I, S1, S2>(mut self, regressand: S1, regressors: I) -> Self
    where
        I: IntoIterator<Item = S2>,
        S1: Into<Cow<'a, str>>,
        S2: Into<Cow<'a, str>>,
    {
        let regressand = regressand.into();
        let regressors: Vec<_> = regressors.into_iter().map(|i| i.into()).collect();
        self.columns = Some((regressand, regressors));
        self
    }

    /// Fits the model and returns a [`RegressionModel`] if successful.
    /// You need to set the data with [`data`] and a formula with [`formula`]
    /// before you can use it.
    ///
    /// [`RegressionModel`]: struct.RegressionModel.html
    /// [`data`]: struct.FormulaRegressionBuilder.html#method.data
    /// [`formula`]: struct.FormulaRegressionBuilder.html#method.formula
    pub fn fit(self) -> Result<RegressionModel, Error> {
        let FittingData(input_vector, output_matrix, outputs) =
            Self::get_matrices_and_regressor_names(self)?;
        RegressionModel::try_from_matrices_and_regressor_names(input_vector, output_matrix, outputs)
    }

    /// Like [`fit`] but does not perfom any statistics on the resulting model.
    /// Returns a [`RegressionParameters`] struct containing the model parameters
    /// if successfull.
    ///
    /// This is usefull if you do not care about the statistics or the model and data
    /// you want to fit result in too few residual degrees of freedom to perform
    /// statistics.
    ///
    /// [`fit`]: struct.FormulaRegressionBuilder.html#method.fit
    /// [`RegressionParameters`]: struct.RegressionParameters.html
    pub fn fit_without_statistics(self) -> Result<RegressionParameters, Error> {
        let FittingData(input_vector, output_matrix, output_names) =
            Self::get_matrices_and_regressor_names(self)?;
        let low_level_result = fit_ols_pinv(input_vector, output_matrix)?;
        let parameters = low_level_result.params;
        let intercept = parameters[0];
        let slopes: Vec<_> = parameters.iter().cloned().skip(1).collect();
        ensure!(
            output_names.len() == slopes.len(),
            Error::InconsistentSlopes(InconsistentSlopes::new(output_names.len(), slopes.len()))
        );
        Ok(RegressionParameters {
            intercept_value: intercept,
            regressor_values: slopes,
            regressor_names: output_names.to_vec(),
        })
    }

    fn get_matrices_and_regressor_names(self) -> Result<FittingData, Error> {
        let (input, outputs) = self.get_data_columns()?;
        let data = &self.data.ok_or(Error::NoData)?.data;
        let input_vector = data
            .get(input.as_ref())
            .ok_or_else(|| Error::ColumnNotInData(input.into()))?;
        let input_vector = RowDVector::from_vec(input_vector.to_vec());
        let mut output_matrix = Vec::new();
        // Add column of all ones as the first column of the matrix
        let all_ones_column = iter::repeat(1.).take(input_vector.len());
        output_matrix.extend(all_ones_column);
        // Add each input as a new column of the matrix
        for output in outputs.to_owned() {
            let output_vec = data
                .get(output.as_ref())
                .ok_or_else(|| Error::ColumnNotInData(output.to_string()))?;
            ensure!(
                output_vec.len() == input_vector.len(),
                Error::RegressorRegressandDimensionMismatch(output.to_string())
            );
            output_matrix.extend(output_vec.iter());
        }
        let output_matrix = DMatrix::from_vec(input_vector.len(), outputs.len() + 1, output_matrix);
        let outputs: Vec<_> = outputs.iter().map(|x| x.to_string()).collect();
        Ok(FittingData(input_vector, output_matrix, outputs))
    }

    fn get_data_columns(&self) -> Result<(Cow<'_, str>, Vec<Cow<'_, str>>), Error> {
        match (self.formula.as_ref(), self.columns.as_ref()) {
            (Some(..), Some(..)) => Err(Error::BothFormulaAndDataColumnsGiven),
            (Some(formula), None) => Self::parse_formula(formula),
            (None, Some((regressand, regressors))) => {
                ensure!(!regressors.is_empty(), Error::InvalidDataColumns);
                Ok((regressand.clone(), regressors.clone()))
            }
            (None, None) => Err(Error::NoFormula),
        }
    }

    fn parse_formula(formula: &str) -> Result<(Cow<'_, str>, Vec<Cow<'_, str>>), Error> {
        let split_formula: Vec<_> = formula.split('~').collect();
        ensure!(split_formula.len() == 2, Error::InvalidFormula);
        let input = split_formula[0].trim();
        let outputs: Vec<_> = split_formula[1]
            .split('+')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(|i| i.into())
            .collect();
        ensure!(!outputs.is_empty(), Error::InvalidFormula);
        Ok((input.into(), outputs))
    }
}

/// A simple tuple struct to reduce the type complxity of the
/// return type of get_matrices_and_regressor_names.
struct FittingData(RowDVector<f64>, DMatrix<f64>, Vec<String>);

/// A container struct for the regression data.
///
/// This struct is obtained using a [`RegressionDataBuilder`].
///
/// [`RegressionDataBuilder`]: struct.RegressionDataBuilder.html
#[derive(Debug, Clone)]
pub struct RegressionData<'a> {
    data: HashMap<Cow<'a, str>, Vec<f64>>,
}

impl<'a> RegressionData<'a> {
    /// Constructs a new `RegressionData` struct from any collection that
    /// implements the `IntoIterator` trait.
    ///
    /// The iterator must consist of tupels of the form `(S, Vec<f64>)` where
    /// `S` is a type that can be converted to a `Cow<'a, str>`.
    ///
    /// `invalid_value_handling` specifies what to do if invalid data is encountered.
    fn new<I, S>(
        data: I,
        invalid_value_handling: InvalidValueHandling,
    ) -> Result<RegressionData<'a>, Error>
    where
        I: IntoIterator<Item = (S, Vec<f64>)>,
        S: Into<Cow<'a, str>>,
    {
        let temp: HashMap<_, _> = data
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect();
        let first_key = temp.keys().next();
        ensure!(
            first_key.is_some(),
            Error::RegressionDataError("The data contains no columns.".into())
        );
        let first_key = first_key.unwrap();
        let first_len = temp[first_key].len();
        ensure!(
            first_len > 0,
            Error::RegressionDataError("The data contains an empty column.".into())
        );
        for key in temp.keys() {
            let this_len = temp[key].len();
            ensure!(
                this_len == first_len,
                Error::RegressionDataError(
                    "The lengths of the columns in the given data are inconsistent.".into()
                )
            );
            ensure!(
                !key.contains('~') && !key.contains('+'),
                Error::RegressionDataError(
                    "The column names may not contain `~` or `+`, because they are used \
                             as separators in the formula."
                        .into()
                )
            );
        }
        if Self::check_if_all_columns_are_equal(&temp) {
            return Err(Error::RegressionDataError(
                "All input columns contain only equal values. Fitting this model would lead \
                     to invalid statistics."
                    .into(),
            ));
        }
        if Self::check_if_data_is_valid(&temp) {
            return Ok(Self { data: temp });
        }
        match invalid_value_handling {
            InvalidValueHandling::ReturnError => Err(Error::RegressionDataError(
                "The data contains a non real value (NaN or infinity or negative infinity). \
                 If you would like to silently drop these values configure the builder with \
                 InvalidValueHandling::DropInvalid."
                    .into(),
            )),
            InvalidValueHandling::DropInvalid => {
                let temp = Self::drop_invalid_values(temp);
                let first_key = temp.keys().next().expect("Cleaned data has no columns.");
                let first_len = temp[first_key].len();
                ensure!(
                    first_len > 0,
                    Error::RegressionDataError("The cleaned data is empty.".into())
                );
                Ok(Self { data: temp })
            }
        }
    }

    fn check_if_all_columns_are_equal(data: &HashMap<Cow<'a, str>, Vec<f64>>) -> bool {
        for column in data.values() {
            if column.is_empty() {
                return false;
            }
            let first_iter = iter::repeat(&column[0]).take(column.len());
            if !first_iter.eq(column.iter()) {
                return false;
            }
        }
        true
    }

    fn check_if_data_is_valid(data: &HashMap<Cow<'a, str>, Vec<f64>>) -> bool {
        for column in data.values() {
            if column.iter().any(|x| !x.is_finite()) {
                return false;
            }
        }
        true
    }

    fn drop_invalid_values(
        data: HashMap<Cow<'a, str>, Vec<f64>>,
    ) -> HashMap<Cow<'a, str>, Vec<f64>> {
        let mut invalid_rows: BTreeSet<usize> = BTreeSet::new();
        for column in data.values() {
            for (index, value) in column.iter().enumerate() {
                if !value.is_finite() {
                    invalid_rows.insert(index);
                }
            }
        }
        let mut cleaned_data = HashMap::new();
        for (key, mut column) in data {
            for index in invalid_rows.iter().rev() {
                column.remove(*index);
            }
            cleaned_data.insert(key, column);
        }
        cleaned_data
    }
}

/// A builder to create a RegressionData struct for use with a [`FormulaRegressionBuilder`].
///
/// [`FormulaRegressionBuilder`]: struct.FormulaRegressionBuilder.html
#[derive(Debug, Clone, Copy)]
pub struct RegressionDataBuilder {
    handle_invalid_values: InvalidValueHandling,
}

impl Default for RegressionDataBuilder {
    fn default() -> RegressionDataBuilder {
        RegressionDataBuilder {
            handle_invalid_values: InvalidValueHandling::default(),
        }
    }
}

impl RegressionDataBuilder {
    /// Create a new [`RegressionDataBuilder`].
    ///
    /// [`RegressionDataBuilder`]: struct.RegressionDataBuilder.html
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure how to handle non real `f64` values (NaN or infinity or negative infinity) using
    /// a variant of the [`InvalidValueHandling`] enum.
    ///
    /// The default value is [`ReturnError`].
    ///
    /// # Example
    /// ```
    /// use linregress::{InvalidValueHandling, RegressionDataBuilder};
    ///
    /// # use linregress::Error;
    /// # fn main() -> Result<(), Error> {
    /// let builder = RegressionDataBuilder::new();
    /// let builder = builder.invalid_value_handling(InvalidValueHandling::DropInvalid);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`InvalidValueHandling`]: enum.InvalidValueHandling.html
    /// [`ReturnError`]: enum.InvalidValueHandling.html#variant.ReturnError
    pub fn invalid_value_handling(mut self, setting: InvalidValueHandling) -> Self {
        self.handle_invalid_values = setting;
        self
    }

    /// Build a [`RegressionData`] struct from the given data.
    ///
    /// Any type that implements the [`IntoIterator`] trait can be used for the data.
    /// This could for example be a [`Hashmap`] or a [`Vec`].
    ///
    /// The iterator must consist of tupels of the form `(S, Vec<f64>)` where
    /// `S` is a type that implements `Into<Cow<str>>`, such as [`String`] or [`str`].
    ///
    /// You can think of this format as the representation of a table of data where
    /// each tuple `(S, Vec<f64>)` represents a column. The `S` is the header or label of the
    /// column and the `Vec<f64>` contains the data of the column.
    ///
    /// Because `~` and `+` are used as separators in the formula they may not be used in the name
    /// of a data column.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use linregress::RegressionDataBuilder;
    ///
    /// # use linregress::Error;
    /// # fn main() -> Result<(), Error> {
    /// let mut data1 = HashMap::new();
    /// data1.insert("Y", vec![1., 2., 3., 4.]);
    /// data1.insert("X", vec![4., 3., 2., 1.]);
    /// let regression_data1 = RegressionDataBuilder::new().build_from(data1)?;
    ///
    /// let y = vec![1., 2., 3., 4.];
    /// let x = vec![4., 3., 2., 1.];
    /// let data2 = vec![("X", x), ("Y", y)];
    /// let regression_data2 = RegressionDataBuilder::new().build_from(data2)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`RegressionData`]: struct.RegressionData.html
    /// [`IntoIterator`]: https://doc.rust-lang.org/std/iter/trait.IntoIterator.html
    /// [`Hashmap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    /// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    /// [`str`]: https://doc.rust-lang.org/std/primitive.str.html
    pub fn build_from<'a, I, S>(self, data: I) -> Result<RegressionData<'a>, Error>
    where
        I: IntoIterator<Item = (S, Vec<f64>)>,
        S: Into<Cow<'a, str>>,
    {
        RegressionData::new(data, self.handle_invalid_values)
    }
}

/// How to proceed if given non real `f64` values (NaN or infinity or negative infinity).
///
/// Used with [`RegressionDataBuilder.invalid_value_handling`]
///
/// The default is [`ReturnError`].
///
/// [`RegressionDataBuilder.invalid_value_handling`]: struct.RegressionDataBuilder.html#method.invalid_value_handling
/// [`ReturnError`]: enum.InvalidValueHandling.html#variant.ReturnError
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum InvalidValueHandling {
    /// Return an error to the caller.
    ReturnError,
    /// Drop the columns containing the invalid values.
    DropInvalid,
}

impl Default for InvalidValueHandling {
    fn default() -> InvalidValueHandling {
        InvalidValueHandling::ReturnError
    }
}

/// A fitted regression model.
///
/// Is the result of [`FormulaRegressionBuilder.fit()`].
///
/// If a field has only one value for the model it is given as `f64`.
///
/// Otherwise it is given as a [`RegressionParameters`] struct.
///
///[`RegressionParameters`]: struct.RegressionParameters.html
///[`FormulaRegressionBuilder.fit()`]: struct.FormulaRegressionBuilder.html#method.fit
#[derive(Debug, Clone)]
pub struct RegressionModel {
    /// The model's intercept and slopes (also known as betas).
    pub parameters: RegressionParameters,
    /// The standard errors of the parameter estimates.
    pub se: RegressionParameters,
    /// Sum of squared residuals.
    pub ssr: f64,
    /// R-squared of the model.
    pub rsquared: f64,
    /// Adjusted R-squared of the model.
    pub rsquared_adj: f64,
    /// The two-tailed p-values for the t-statistics of the params.
    pub pvalues: RegressionParameters,
    /// The residuals of the model.
    pub residuals: RegressionParameters,
    ///  A scale factor for the covariance matrix.
    ///
    ///  Note that the square root of `scale` is often
    ///  called the standard error of the regression.
    pub scale: f64,
}

impl RegressionModel {
    /// Evaluates the model on given new input data and returns the predicted values.
    ///
    /// The new data is expected to have the same columns as the original data.
    /// See [`RegressionDataBuilder.build`] for details on the type of the `new_data` parameter.
    ///
    /// ## Note
    ///
    /// This function does *no* special handling of non real values (NaN or infinity or negative infinity).
    /// Such a value in `new_data` will result in a corresponding meaningless prediction.
    ///
    /// ## Example
    ///
    /// ```
    /// # use linregress::{RegressionDataBuilder, FormulaRegressionBuilder};
    /// # use linregress::Error;
    /// # fn main() -> Result<(), Error> {
    /// let y = vec![1., 2., 3., 4., 5.];
    /// let x1 = vec![5., 4., 3., 2., 1.];
    /// let x2 = vec![729.53, 439.0367, 42.054, 1., 0.];
    /// let x3 = vec![258.589, 616.297, 215.061, 498.361, 0.];
    /// let data = vec![("Y", y), ("X1", x1), ("X2", x2), ("X3", x3)];
    /// let data = RegressionDataBuilder::new().build_from(data).unwrap();
    /// let formula = "Y ~ X1 + X2 + X3";
    /// let model = FormulaRegressionBuilder::new()
    ///     .data(&data)
    ///     .formula(formula)
    ///     .fit()?;
    /// let new_data = vec![
    ///     ("X1", vec![2.5, 3.5]),
    ///     ("X2", vec![2.0, 8.0]),
    ///     ("X3", vec![2.0, 1.0]),
    /// ];
    /// let prediction: Vec<f64> = model.predict(new_data)?;
    /// assert_eq!(prediction, vec![3.5000000000000275, 2.5000000000000533]);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`RegressionDataBuilder.build`]: struct.RegressionDataBuilder.html#method.build_from
    pub fn predict<'a, I, S>(&self, new_data: I) -> Result<Vec<f64>, Error>
    where
        I: IntoIterator<Item = (S, Vec<f64>)>,
        S: Into<Cow<'a, str>>,
    {
        let new_data: HashMap<Cow<'_, _>, Vec<f64>> = new_data
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect();
        self.check_variables(&new_data)?;
        let input_len = new_data.values().next().unwrap().len();
        let mut new_data_values: Vec<f64> = vec![];
        for key in &self.parameters.regressor_names {
            new_data_values.extend_from_slice(new_data[&Cow::from(key)].as_slice());
        }
        let new_data_matrix = DMatrix::from_vec(
            input_len,
            self.parameters.regressor_values.len(),
            new_data_values,
        );
        let param_matrix = DMatrix::from_iterator(
            self.parameters.regressor_values.len(),
            1,
            self.parameters.regressor_values.iter().copied(),
        );
        let intercept = self.parameters.intercept_value;
        let intercept_matrix =
            DMatrix::from_iterator(input_len, 1, std::iter::repeat(intercept).take(input_len));
        let predictions = (new_data_matrix * param_matrix) + intercept_matrix;
        let predictions: Vec<f64> = predictions.into_iter().copied().collect();
        Ok(predictions)
    }

    fn check_variables<'a>(
        &self,
        given_parameters: &HashMap<Cow<'a, str>, Vec<f64>>,
    ) -> Result<(), Error> {
        ensure!(!given_parameters.is_empty(), Error::NoData);
        let first_len = given_parameters.values().next().unwrap().len();
        ensure!(first_len > 0, Error::NoData);
        ensure!(
            self.parameters.regressor_names.len() == self.parameters.regressor_values.len(),
            Error::InconsistentRegressionModel
        );
        let model_parameters: HashSet<_> = self
            .parameters
            .regressor_names
            .iter()
            .map(Cow::from)
            .collect();
        for param in &model_parameters {
            if !given_parameters.contains_key(param) {
                return Err(Error::ColumnNotInData(param.to_string()));
            }
        }
        for (param, values) in given_parameters {
            ensure!(values.len() == first_len, Error::InconsistentVectors);
            if !model_parameters.contains(param) {
                return Err(Error::ModelColumnNotInData(param.to_string()));
            }
        }
        Ok(())
    }

    fn try_from_matrices_and_regressor_names<I: IntoIterator<Item = String>>(
        inputs: RowDVector<f64>,
        outputs: DMatrix<f64>,
        output_names: I,
    ) -> Result<Self, Error> {
        let low_level_result = fit_ols_pinv(inputs.to_owned(), outputs.to_owned())?;
        let LowLevelRegressionModel {
            parameters,
            se,
            ssr,
            rsquared,
            rsquared_adj,
            pvalues,
            residuals,
            scale,
        } = LowLevelRegressionModel::from_low_level_regression(low_level_result)?;
        let output_names: Vec<_> = output_names.into_iter().collect();
        let intercept = parameters[0];
        let slopes: Vec<f64> = parameters.into_iter().skip(1).collect();
        ensure!(
            output_names.len() == slopes.len(),
            Error::InconsistentSlopes(InconsistentSlopes::new(output_names.len(), slopes.len(),))
        );
        let parameters = RegressionParameters {
            intercept_value: intercept,
            regressor_values: slopes,
            regressor_names: output_names.to_vec(),
        };
        let se = RegressionParameters {
            intercept_value: se[0],
            regressor_values: se.iter().cloned().skip(1).collect(),
            regressor_names: output_names.to_vec(),
        };
        let residuals = RegressionParameters {
            intercept_value: residuals[0],
            regressor_values: residuals.iter().cloned().skip(1).collect(),
            regressor_names: output_names.to_vec(),
        };
        let pvalues = RegressionParameters {
            intercept_value: pvalues[0],
            regressor_values: pvalues.iter().cloned().skip(1).collect(),
            regressor_names: output_names.to_vec(),
        };
        Ok(Self {
            parameters,
            se,
            ssr,
            rsquared,
            rsquared_adj,
            pvalues,
            residuals,
            scale,
        })
    }
}

/// A fitted regression model
///
/// Is the result of [`fit_low_level_regression_model`].
///
/// If a field has only one value for the model it is given as `f64`.
///
/// Otherwise it  is given as a `Vec<f64>` where the first value is the intercept value.
#[derive(Debug, Clone)]
pub struct LowLevelRegressionModel {
    /// The model's intercept and slopes (also known as betas).
    pub parameters: Vec<f64>,
    /// The standard errors of the parameter estimates.
    pub se: Vec<f64>,
    /// Sum of squared residuals.
    pub ssr: f64,
    /// R-squared of the model.
    pub rsquared: f64,
    /// Adjusted R-squared of the model.
    pub rsquared_adj: f64,
    /// The two-tailed p-values for the t-statistics of the params.
    pub pvalues: Vec<f64>,
    /// The residuals of the model.
    pub residuals: Vec<f64>,
    ///  A scale factor for the covariance matrix.
    ///
    ///  Note that the square root of `scale` is often
    ///  called the standard error of the regression.
    pub scale: f64,
}

impl LowLevelRegressionModel {
    fn from_low_level_regression(
        low_level_result: InternalLowLevelRegressionResult,
    ) -> Result<Self, Error> {
        let parameters = low_level_result.params;
        let singular_values = low_level_result.singular_values;
        let normalized_cov_params = low_level_result.normalized_cov_params;
        let diag = DMatrix::from_diagonal(&singular_values);
        let rank = &diag.rank(0.0);
        let input_vec: Vec<_> = low_level_result.inputs.iter().copied().collect();
        let input_matrix = DMatrix::from_vec(low_level_result.inputs.len(), 1, input_vec);
        let residuals = &input_matrix - (low_level_result.outputs * parameters.to_owned());
        let ssr = residuals.dot(&residuals);
        let n = low_level_result.inputs.ncols();
        let df_resid = n - rank;
        ensure!(
            df_resid >= 1,
            Error::ModelFittingError(
                "There are not enough residual degrees of freedom to perform statistics on this model".into()));
        let scale = residuals.dot(&residuals) / df_resid as f64;
        let cov_params = normalized_cov_params * scale;
        let se = get_se_from_cov_params(&cov_params)?;
        let centered_input_matrix = subtract_value_from_matrix(&input_matrix, input_matrix.mean());
        let centered_tss = &centered_input_matrix.dot(&centered_input_matrix);
        let rsquared = 1. - (ssr / centered_tss);
        let rsquared_adj = 1. - ((n - 1) as f64 / df_resid as f64 * (1. - rsquared));
        let tvalues: Vec<_> = parameters
            .iter()
            .zip(se.iter())
            .map(|(x, y)| x / y)
            .collect();
        let pvalues: Vec<_> = tvalues
            .iter()
            .cloned()
            .map(|x| stdtr(df_resid as i64, -(x.abs())) * 2.)
            .collect();
        // Convert these from interal Matrix types to user facing types
        let parameters: Vec<f64> = parameters.iter().copied().collect();
        let se: Vec<f64> = se.iter().copied().collect();
        let residuals: Vec<f64> = residuals.iter().copied().collect();
        Ok(Self {
            parameters,
            se,
            ssr,
            rsquared,
            rsquared_adj,
            pvalues,
            residuals,
            scale,
        })
    }
}

/// A parameter of a fitted [`RegressionModel`] given for the intercept and each regressor.
///
/// The values and names of the regressors are given in the same order.
///
/// You can obtain name value pairs using [`pairs`].
///
/// [`RegressionModel`]: struct.RegressionModel.html
/// [`pairs`]: struct.RegressionParameters.html#method.pairs
#[derive(Debug, Clone)]
pub struct RegressionParameters {
    pub intercept_value: f64,
    pub regressor_names: Vec<String>,
    pub regressor_values: Vec<f64>,
}

impl RegressionParameters {
    /// Returns the parameters as a Vec of tuples of the form `(name: &str, value: f64)`.
    ///
    /// # Usage
    ///
    /// ```
    /// use linregress::{FormulaRegressionBuilder, RegressionDataBuilder};
    ///
    /// # use linregress::Error;
    /// # fn main() -> Result<(), Error> {
    /// let y = vec![1.,2. ,3. , 4.];
    /// let x1 = vec![4., 3., 2., 1.];
    /// let x2 = vec![1., 2., 3., 4.];
    /// let data = vec![("Y", y), ("X1", x1), ("X2", x2)];
    /// let data = RegressionDataBuilder::new().build_from(data)?;
    /// let model = FormulaRegressionBuilder::new().data(&data).formula("Y ~ X1 + X2").fit()?;
    /// let pairs = model.parameters.pairs();
    /// assert_eq!(pairs[0], ("X1", -0.0370370370370372));
    /// assert_eq!(pairs[1], ("X2", 0.9629629629629629));
    /// # Ok(())
    /// # }
    /// ```
    pub fn pairs(&self) -> Vec<(&str, f64)> {
        self.regressor_names
            .iter()
            .zip(self.regressor_values.iter())
            .map(|(x, y)| (x.as_str(), *y))
            .collect()
    }
}

/// Fit a regression model directly on a matrix of input data
///
/// Expects a matrix in the format
///
/// | regressand | intercept | regressor 1 | regressor 2 | …   |
/// |------------|-----------|-------------|-------------|-----|
/// | value      | 1.0       | value       | value       | …   |
/// | ⋮          | ⋮         | ⋮           | ⋮           | ⋮   |
///
/// in row major order.
///
/// # Note
/// - The matrix should already contain the `intercept` column consisting of only the value `1.0`.
/// - No validation of the data is performed, except for a simple dimension consistency check.
///
/// # Example
/// ```
/// # fn main() -> Result<(), linregress::Error> {
/// use linregress::fit_low_level_regression_model;
///
/// let data_row_major: Vec<f64> = vec![
///     1., 1.0, 1., 7.,
///     3., 1.0, 2., 6.,
///     4., 1.0, 3., 5.,
///     5., 1.0, 4., 4.,
///     2., 1.0, 5., 3.,
///     3., 1.0, 6., 2.,
///     4., 1.0, 7., 1.,
/// ];
/// let model = fit_low_level_regression_model(&data_row_major, 7, 4)?;
/// let params = [
///     0.09523809523809511f64,
///     0.5059523809523809,
///     0.25595238095238104,
/// ];
/// assert_eq!(model.parameters, params);
/// # Ok(())
/// # }
/// ```
pub fn fit_low_level_regression_model(
    data_row_major: &[f64],
    num_rows: usize,
    num_columns: usize,
) -> Result<LowLevelRegressionModel, Error> {
    let regression = get_low_level_regression(data_row_major, num_rows, num_columns)?;
    let model = LowLevelRegressionModel::from_low_level_regression(regression)?;
    Ok(model)
}

/// Like [`fit_low_level_regression_model`] but does not compute any statistics after
/// fitting the model.
///
/// Returns a `Vec<f64>` analogous to the `parameters` field of [`LowLevelRegressionModel`].
pub fn fit_low_level_regression_model_without_statistics(
    data_row_major: &[f64],
    num_rows: usize,
    num_columns: usize,
) -> Result<Vec<f64>, Error> {
    let regression = get_low_level_regression(data_row_major, num_rows, num_columns)?;
    Ok(regression.params.iter().copied().collect())
}

fn get_low_level_regression(
    data_row_major: &[f64],
    num_rows: usize,
    num_columns: usize,
) -> Result<InternalLowLevelRegressionResult, Error> {
    ensure!(
        !data_row_major.is_empty() && num_rows * num_columns == data_row_major.len(),
        Error::InconsistentVectors
    );
    let data = DMatrix::from_row_slice(num_rows, num_columns, data_row_major);
    let inputs = data.slice((0, 0), (num_rows, 1));
    let inputs: RowDVector<f64> = RowDVector::from_iterator(num_rows, inputs.iter().copied());
    let outputs: DMatrix<f64> = data.slice((0, 1), (num_rows, num_columns - 1)).into_owned();
    fit_ols_pinv(inputs, outputs)
}

/// Result of fitting a low level matrix based model
#[derive(Debug, Clone)]
struct InternalLowLevelRegressionResult {
    inputs: RowDVector<f64>,
    outputs: DMatrix<f64>,
    params: DMatrix<f64>,
    singular_values: DVector<f64>,
    normalized_cov_params: DMatrix<f64>,
}

/// Performs ordinary least squared linear regression using the pseudo inverse method.
///
/// Returns a tuple `LowLevelRegressionResult`
fn fit_ols_pinv(
    inputs: RowDVector<f64>,
    outputs: DMatrix<f64>,
) -> Result<InternalLowLevelRegressionResult, Error> {
    ensure!(
        !inputs.is_empty(),
        Error::ModelFittingError(
            "Fitting the model failed because the input vector is empty".into()
        )
    );
    ensure!(
        outputs.nrows() >= 1 && outputs.ncols() >= 1,
        Error::ModelFittingError(
            "Fitting the model failed because the output matrix is empty".into()
        )
    );
    let singular_values = outputs
        .to_owned()
        .try_svd(false, false, std::f64::EPSILON, 0)
        .ok_or_else(|| {
            Error::ModelFittingError(
                "computing the singular-value decomposition of the output matrix failed".into(),
            )
        })?
        .singular_values;
    let pinv = outputs.clone().pseudo_inverse(0.).map_err(|_| {
        Error::ModelFittingError("Taking the pinv of the output matrix failed".into())
    });
    let pinv = pinv?;
    let normalized_cov_params = &pinv * &pinv.transpose();
    let params = get_sum_of_products(&pinv, &inputs);
    ensure!(
        params.len() >= 2,
        Error::ModelFittingError("Invalid parameter matrix".into())
    );
    Ok(InternalLowLevelRegressionResult {
        inputs,
        outputs,
        params,
        singular_values,
        normalized_cov_params,
    })
}

/// Subtracts `value` from all fields in `matrix` and returns the resulting new matrix.
fn subtract_value_from_matrix(matrix: &DMatrix<f64>, value: f64) -> DMatrix<f64> {
    let nrows = matrix.nrows();
    let ncols = matrix.ncols();
    let substraction_matrix: DMatrix<f64> =
        DMatrix::from_iterator(nrows, ncols, std::iter::repeat(value).take(nrows * ncols));
    matrix - substraction_matrix
}

/// Calculates the standard errors given a model's covariate parameters
fn get_se_from_cov_params(matrix: &DMatrix<f64>) -> Result<DMatrix<f64>, Error> {
    let mut v = Vec::new();
    for row_index in 0..matrix.ncols() {
        let row = matrix.row(row_index);
        ensure!(
            row_index <= row.len(),
            Error::ModelFittingError("Matrix is not square".into())
        );
        v.push(row[row_index].sqrt());
    }
    Ok(DMatrix::from_vec(matrix.ncols(), 1, v))
}

fn get_sum_of_products(matrix: &DMatrix<f64>, vector: &RowDVector<f64>) -> DMatrix<f64> {
    let mut v: Vec<f64> = Vec::new();
    for row_index in 0..matrix.nrows() {
        let row = matrix.row(row_index);
        let mut sum = 0.;
        for (x, y) in row.iter().zip(vector.iter()) {
            sum += x * y;
        }
        v.push(sum);
    }
    DMatrix::from_vec(matrix.nrows(), 1, v)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn assert_almost_equal(a: f64, b: f64) {
        assert_almost_equal_with_precision(a, b, 1.0E-14);
    }

    fn assert_almost_equal_with_precision(a: f64, b: f64, precision: f64) {
        if (a - b).abs() > precision {
            panic!("\n{:?} vs\n{:?}", a, b);
        }
    }

    fn assert_slices_almost_equal(a: &[f64], b: &[f64]) {
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().cloned().zip(b.iter().cloned()).collect::<Vec<_>>() {
            assert_almost_equal(x, y);
        }
    }

    #[test]
    fn test_pinv_with_formula_builder() {
        use std::collections::HashMap;
        let inputs = vec![1., 3., 4., 5., 2., 3., 4.];
        let outputs1 = vec![1., 2., 3., 4., 5., 6., 7.];
        let outputs2 = vec![7., 6., 5., 4., 3., 2., 1.];
        let mut data = HashMap::new();
        data.insert("Y", inputs);
        data.insert("X1", outputs1);
        data.insert("X2", outputs2);
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        let regression = FormulaRegressionBuilder::new()
            .data(&data)
            .formula("Y ~ X1 + X2")
            .fit()
            .expect("Fitting model failed");

        let model_parameters = vec![0.09523809523809523, 0.5059523809523809, 0.2559523809523808];
        let se = vec![
            0.015457637291218289,
            0.1417242813072997,
            0.14172428130729975,
        ];
        let ssr = 9.107142857142858;
        let rsquared = 0.16118421052631582;
        let rsquared_adj = -0.006578947368421018;
        let scale = 1.8214285714285716;
        let pvalues = vec![
            0.001639031204417556,
            0.016044083709847945,
            0.13074580446389245,
        ];
        let residuals = vec![
            -1.392857142857142,
            0.3571428571428581,
            1.1071428571428577,
            1.8571428571428577,
            -1.3928571428571423,
            -0.6428571428571423,
            0.10714285714285765,
        ];
        assert_almost_equal(regression.parameters.intercept_value, model_parameters[0]);
        assert_almost_equal(
            regression.parameters.regressor_values[0],
            model_parameters[1],
        );
        assert_almost_equal(
            regression.parameters.regressor_values[1],
            model_parameters[2],
        );
        assert_almost_equal(regression.se.intercept_value, se[0]);
        assert_slices_almost_equal(&regression.se.regressor_values, &se[1..]);
        assert_almost_equal(regression.ssr, ssr);
        assert_almost_equal(regression.rsquared, rsquared);
        assert_almost_equal(regression.rsquared_adj, rsquared_adj);
        assert_almost_equal(regression.pvalues.intercept_value, pvalues[0]);
        assert_slices_almost_equal(&regression.pvalues.regressor_values, &pvalues[1..]);
        assert_almost_equal(regression.residuals.intercept_value, residuals[0]);
        assert_slices_almost_equal(&regression.residuals.regressor_values, &residuals[1..]);
        assert_eq!(regression.scale, scale);
    }

    #[test]
    fn test_pinv_with_data_columns() {
        use std::collections::HashMap;
        let inputs = vec![1., 3., 4., 5., 2., 3., 4.];
        let outputs1 = vec![1., 2., 3., 4., 5., 6., 7.];
        let outputs2 = vec![7., 6., 5., 4., 3., 2., 1.];
        let mut data = HashMap::new();
        data.insert("Y", inputs);
        data.insert("X1", outputs1);
        data.insert("X2", outputs2);
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        let regression = FormulaRegressionBuilder::new()
            .data(&data)
            .data_columns("Y", ["X1", "X2"])
            .fit()
            .expect("Fitting model failed");

        let model_parameters = vec![0.09523809523809523, 0.5059523809523809, 0.2559523809523808];
        let se = vec![
            0.015457637291218289,
            0.1417242813072997,
            0.14172428130729975,
        ];
        let ssr = 9.107142857142858;
        let rsquared = 0.16118421052631582;
        let rsquared_adj = -0.006578947368421018;
        let scale = 1.8214285714285716;
        let pvalues = vec![
            0.001639031204417556,
            0.016044083709847945,
            0.13074580446389245,
        ];
        let residuals = vec![
            -1.392857142857142,
            0.3571428571428581,
            1.1071428571428577,
            1.8571428571428577,
            -1.3928571428571423,
            -0.6428571428571423,
            0.10714285714285765,
        ];
        assert_almost_equal(regression.parameters.intercept_value, model_parameters[0]);
        assert_almost_equal(
            regression.parameters.regressor_values[0],
            model_parameters[1],
        );
        assert_almost_equal(
            regression.parameters.regressor_values[1],
            model_parameters[2],
        );
        assert_almost_equal(regression.se.intercept_value, se[0]);
        assert_slices_almost_equal(&regression.se.regressor_values, &se[1..]);
        assert_almost_equal(regression.ssr, ssr);
        assert_almost_equal(regression.rsquared, rsquared);
        assert_almost_equal(regression.rsquared_adj, rsquared_adj);
        assert_almost_equal(regression.pvalues.intercept_value, pvalues[0]);
        assert_slices_almost_equal(&regression.pvalues.regressor_values, &pvalues[1..]);
        assert_almost_equal(regression.residuals.intercept_value, residuals[0]);
        assert_slices_almost_equal(&regression.residuals.regressor_values, &residuals[1..]);
        assert_eq!(regression.scale, scale);
    }

    #[test]
    fn test_low_level_model_fitting() {
        let inputs = vec![1., 3., 4., 5., 2., 3., 4.];
        let outputs1 = vec![1., 2., 3., 4., 5., 6., 7.];
        let outputs2 = vec![7., 6., 5., 4., 3., 2., 1.];
        let mut data_row_major = Vec::with_capacity(4 * 7);
        for n in 0..7 {
            data_row_major.push(inputs[n]);
            data_row_major.push(1.0);
            data_row_major.push(outputs1[n]);
            data_row_major.push(outputs2[n]);
        }
        let regression = fit_low_level_regression_model(&data_row_major, 7, 4).unwrap();
        let model_parameters = vec![0.09523809523809523, 0.5059523809523809, 0.2559523809523808];
        let se = vec![
            0.015457637291218289,
            0.1417242813072997,
            0.14172428130729975,
        ];
        let ssr = 9.107142857142858;
        let rsquared = 0.16118421052631582;
        let rsquared_adj = -0.006578947368421018;
        let scale = 1.8214285714285716;
        let pvalues = vec![
            0.001639031204417556,
            0.016044083709847945,
            0.13074580446389245,
        ];
        let residuals = vec![
            -1.392857142857142,
            0.3571428571428581,
            1.1071428571428577,
            1.8571428571428577,
            -1.3928571428571423,
            -0.6428571428571423,
            0.10714285714285765,
        ];
        assert_slices_almost_equal(&regression.parameters, &model_parameters);
        assert_slices_almost_equal(&regression.se, &se);
        assert_almost_equal(regression.ssr, ssr);
        assert_almost_equal(regression.rsquared, rsquared);
        assert_almost_equal(regression.rsquared_adj, rsquared_adj);
        assert_slices_almost_equal(&regression.pvalues, &pvalues);
        assert_slices_almost_equal(&regression.residuals, &residuals);
        assert_eq!(regression.scale, scale);
    }

    #[test]
    fn test_without_statistics() {
        use std::collections::HashMap;
        let inputs = vec![1., 3., 4., 5., 2., 3., 4.];
        let outputs1 = vec![1., 2., 3., 4., 5., 6., 7.];
        let outputs2 = vec![7., 6., 5., 4., 3., 2., 1.];
        let mut data = HashMap::new();
        data.insert("Y", inputs);
        data.insert("X1", outputs1);
        data.insert("X2", outputs2);
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        let regression = FormulaRegressionBuilder::new()
            .data(&data)
            .formula("Y ~ X1 + X2")
            .fit_without_statistics()
            .expect("Fitting model failed");
        let model_parameters = vec![0.09523809523809523, 0.5059523809523809, 0.2559523809523808];
        assert_almost_equal(regression.intercept_value, model_parameters[0]);
        assert_almost_equal(regression.regressor_values[0], model_parameters[1]);
        assert_almost_equal(regression.regressor_values[1], model_parameters[2]);
    }

    #[test]
    fn test_invalid_input_empty_matrix() {
        let y = vec![];
        let x1 = vec![];
        let x2 = vec![];
        let data = vec![("Y", y), ("X1", x1), ("X2", x2)];
        let data = RegressionDataBuilder::new().build_from(data);
        assert!(data.is_err());
    }

    #[test]
    fn test_invalid_input_wrong_shape_x() {
        let y = vec![1., 2., 3.];
        let x1 = vec![1., 2., 3.];
        let x2 = vec![1., 2.];
        let data = vec![("Y", y), ("X1", x1), ("X2", x2)];
        let data = RegressionDataBuilder::new().build_from(data);
        assert!(data.is_err());
    }

    #[test]
    fn test_invalid_input_wrong_shape_y() {
        let y = vec![1., 2., 3., 4.];
        let x1 = vec![1., 2., 3.];
        let x2 = vec![1., 2., 3.];
        let data = vec![("Y", y), ("X1", x1), ("X2", x2)];
        let data = RegressionDataBuilder::new().build_from(data);
        assert!(data.is_err());
    }

    #[test]
    fn test_invalid_input_nan() {
        let y1 = vec![1., 2., 3., 4.];
        let x1 = vec![1., 2., 3., std::f64::NAN];
        let data1 = vec![("Y", y1), ("X", x1)];
        let y2 = vec![1., 2., 3., std::f64::NAN];
        let x2 = vec![1., 2., 3., 4.];
        let data2 = vec![("Y", y2), ("X", x2)];
        let r_data1 = RegressionDataBuilder::new().build_from(data1.to_owned());
        let r_data2 = RegressionDataBuilder::new().build_from(data2.to_owned());
        assert!(r_data1.is_err());
        assert!(r_data2.is_err());
        let builder = RegressionDataBuilder::new();
        let builder = builder.invalid_value_handling(InvalidValueHandling::DropInvalid);
        let r_data1 = builder.build_from(data1);
        let r_data2 = builder.build_from(data2);
        assert!(r_data1.is_ok());
        assert!(r_data2.is_ok());
    }

    #[test]
    fn test_invalid_input_infinity() {
        let y1 = vec![1., 2., 3., 4.];
        let x1 = vec![1., 2., 3., std::f64::INFINITY];
        let data1 = vec![("Y", y1), ("X", x1)];
        let y2 = vec![1., 2., 3., std::f64::NEG_INFINITY];
        let x2 = vec![1., 2., 3., 4.];
        let data2 = vec![("Y", y2), ("X", x2)];
        let r_data1 = RegressionDataBuilder::new().build_from(data1.to_owned());
        let r_data2 = RegressionDataBuilder::new().build_from(data2.to_owned());
        assert!(r_data1.is_err());
        assert!(r_data2.is_err());
        let builder = RegressionDataBuilder::new();
        let builder = builder.invalid_value_handling(InvalidValueHandling::DropInvalid);
        let r_data1 = builder.build_from(data1);
        let r_data2 = builder.build_from(data2);
        assert!(r_data1.is_ok());
        assert!(r_data2.is_ok());
    }

    #[test]
    fn test_invalid_input_all_equal_columns() {
        let y = vec![38.0, 38.0, 38.0];
        let x = vec![42.0, 42.0, 42.0];
        let data = vec![("y", y), ("x", x)];
        let data = RegressionDataBuilder::new().build_from(data);
        assert!(data.is_err());
    }

    #[test]
    fn test_drop_invalid_values() {
        let mut data: HashMap<Cow<'_, str>, Vec<f64>> = HashMap::new();
        data.insert("Y".into(), vec![-1., -2., -3., -4.]);
        data.insert("foo".into(), vec![1., 2., 12., 4.]);
        data.insert("bar".into(), vec![1., 1., 7., 4.]);
        data.insert("baz".into(), vec![1.3333, 2.754, 3.12, 4.11]);
        assert_eq!(RegressionData::drop_invalid_values(data.to_owned()), data);
        data.insert(
            "invalid".into(),
            vec![std::f64::NAN, 42., std::f64::NEG_INFINITY, 23.11],
        );
        data.insert(
            "invalid2".into(),
            vec![1.337, -3.14, std::f64::INFINITY, 11.111111],
        );
        let mut ref_data: HashMap<Cow<'_, str>, Vec<f64>> = HashMap::new();
        ref_data.insert("Y".into(), vec![-2., -4.]);
        ref_data.insert("foo".into(), vec![2., 4.]);
        ref_data.insert("bar".into(), vec![1., 4.]);
        ref_data.insert("baz".into(), vec![2.754, 4.11]);
        ref_data.insert("invalid".into(), vec![42., 23.11]);
        ref_data.insert("invalid2".into(), vec![-3.14, 11.111111]);
        assert_eq!(
            ref_data,
            RegressionData::drop_invalid_values(data.to_owned())
        );
    }

    #[test]
    fn test_all_invalid_input() {
        let data = vec![
            ("Y", vec![1., 2., 3.]),
            ("X", vec![std::f64::NAN, std::f64::NAN, std::f64::NAN]),
        ];
        let builder = RegressionDataBuilder::new();
        let builder = builder.invalid_value_handling(InvalidValueHandling::DropInvalid);
        let r_data = builder.build_from(data);
        assert!(r_data.is_err());
    }

    #[test]
    fn test_invalid_column_names() {
        let data1 = vec![("x~f", vec![1., 2., 3.]), ("foo", vec![0., 0., 0.])];
        let data2 = vec![("foo", vec![1., 2., 3.]), ("foo+", vec![0., 0., 0.])];
        let builder = RegressionDataBuilder::new();
        assert!(builder.build_from(data1).is_err());
        assert!(builder.build_from(data2).is_err());
    }

    #[test]
    fn test_no_formula() {
        let data = vec![("x", vec![1., 2., 3.]), ("foo", vec![0., 0., 0.])];
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        let res = FormulaRegressionBuilder::new().data(&data).fit();
        assert!(res.is_err());
    }

    #[test]
    fn test_both_formula_and_data_columns() {
        let y = vec![1., 2., 3., 4., 5.];
        let x1 = vec![5., 4., 3., 2., 1.];
        let x2 = vec![729.53, 439.0367, 42.054, 1., 0.];
        let x3 = vec![258.589, 616.297, 215.061, 498.361, 0.];
        let data = vec![("Y", y), ("X1", x1), ("X2", x2), ("X3", x3)];
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        let formula = "Y ~ X1 + X2 + X3";
        let res = FormulaRegressionBuilder::new()
            .data(&data)
            .formula(formula)
            .data_columns("Y", ["X1", "X2", "X3"])
            .fit();
        assert!(res.is_err());
    }

    fn build_model() -> RegressionModel {
        let y = vec![1., 2., 3., 4., 5.];
        let x1 = vec![5., 4., 3., 2., 1.];
        let x2 = vec![729.53, 439.0367, 42.054, 1., 0.];
        let x3 = vec![258.589, 616.297, 215.061, 498.361, 0.];
        let data = vec![("Y", y), ("X1", x1), ("X2", x2), ("X3", x3)];
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        let formula = "Y ~ X1 + X2 + X3";
        FormulaRegressionBuilder::new()
            .data(&data)
            .formula(formula)
            .fit()
            .unwrap()
    }

    #[test]
    fn test_prediction_empty_vectors() {
        let model = build_model();
        let new_data: HashMap<Cow<'_, _>, _> = vec![("X1", vec![]), ("X2", vec![]), ("X3", vec![])]
            .into_iter()
            .map(|(x, y)| (Cow::from(x), y))
            .collect();
        assert!(model.check_variables(&new_data).is_err());
    }

    #[test]
    fn test_prediction_vectors_with_different_lengths() {
        let model = build_model();
        let new_data: HashMap<Cow<'_, _>, _> = vec![
            ("X1", vec![1.0, 2.0]),
            ("X2", vec![2.0, 1.0]),
            ("X3", vec![3.0]),
        ]
        .into_iter()
        .map(|(x, y)| (Cow::from(x), y))
        .collect();
        assert!(model.check_variables(&new_data).is_err());
    }

    #[test]
    fn test_too_many_prediction_variables() {
        let model = build_model();
        let new_data: HashMap<Cow<'_, _>, _> = vec![
            ("X1", vec![1.0]),
            ("X2", vec![2.0]),
            ("X3", vec![3.0]),
            ("X4", vec![4.0]),
        ]
        .into_iter()
        .map(|(x, y)| (Cow::from(x), y))
        .collect();
        assert!(model.check_variables(&new_data).is_err());
    }

    #[test]
    fn test_not_enough_prediction_variables() {
        let model = build_model();
        let new_data: HashMap<Cow<'_, _>, _> = vec![("X1", vec![1.0]), ("X2", vec![2.0])]
            .into_iter()
            .map(|(x, y)| (Cow::from(x), y))
            .collect();
        assert!(model.check_variables(&new_data).is_err());
    }

    #[test]
    fn test_prediction_broken_model() {
        let mut model = build_model();
        model.parameters.regressor_values = vec![];
        let new_data: HashMap<Cow<'_, _>, _> =
            vec![("X1", vec![1.0]), ("X2", vec![2.0]), ("X3", vec![3.0])]
                .into_iter()
                .map(|(x, y)| (Cow::from(x), y))
                .collect();
        assert!(model.check_variables(&new_data).is_err());
    }

    #[test]
    fn test_prediction() {
        let model = build_model();
        let new_data = vec![("X1", vec![2.5]), ("X2", vec![2.0]), ("X3", vec![2.0])];
        let prediction = model.predict(new_data).unwrap();
        assert_eq!(prediction.len(), 1);
        assert_almost_equal_with_precision(prediction[0], 3.500000000000111, 1.0E-7);
    }

    #[test]
    fn test_multiple_predictions() {
        let model = build_model();
        let new_data = vec![
            ("X1", vec![2.5, 3.5]),
            ("X2", vec![2.0, 8.0]),
            ("X3", vec![2.0, 1.0]),
        ];
        let prediction = model.predict(new_data).unwrap();
        assert_eq!(prediction.len(), 2);
        assert_almost_equal_with_precision(prediction[0], 3.500000000000111, 1.0E-7);
        assert_almost_equal_with_precision(prediction[1], 2.5000000000001337, 1.0E-7);
    }

    #[test]
    fn test_multiple_predictions_out_of_order() {
        let model = build_model();
        let new_data = vec![
            ("X1", vec![2.5, 3.5]),
            ("X3", vec![2.0, 1.0]),
            ("X2", vec![2.0, 8.0]),
        ];
        let prediction = model.predict(new_data).unwrap();
        assert_eq!(prediction.len(), 2);
        assert_almost_equal_with_precision(prediction[0], 3.500000000000111, 1.0E-7);
        assert_almost_equal_with_precision(prediction[1], 2.5000000000001337, 1.0E-7);
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    use super::*;
    extern crate test;
    use test::Bencher;

    #[bench]
    fn bench_with_stats(b: &mut Bencher) {
        let y = vec![1., 2., 3., 4., 5.];
        let x1 = vec![5., 4., 3., 2., 1.];
        let x2 = vec![729.53, 439.0367, 42.054, 1., 0.];
        let x3 = vec![258.589, 616.297, 215.061, 498.361, 0.];
        let data = vec![("Y", y), ("X1", x1), ("X2", x2), ("X3", x3)];
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        let formula = "Y ~ X1 + X2 + X3";
        b.iter(|| {
            FormulaRegressionBuilder::new()
                .data(&data)
                .formula(formula)
                .fit()
        });
    }

    #[bench]
    fn bench_data_columns_with_stats(b: &mut Bencher) {
        let y = vec![1., 2., 3., 4., 5.];
        let x1 = vec![5., 4., 3., 2., 1.];
        let x2 = vec![729.53, 439.0367, 42.054, 1., 0.];
        let x3 = vec![258.589, 616.297, 215.061, 498.361, 0.];
        let data = vec![("Y", y), ("X1", x1), ("X2", x2), ("X3", x3)];
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        b.iter(|| {
            FormulaRegressionBuilder::new()
                .data(&data)
                .data_columns("Y", ["X1", "X2", "X3"])
                .fit()
        });
    }

    #[bench]
    fn bench_without_stats(b: &mut Bencher) {
        let y = vec![1., 2., 3., 4., 5.];
        let x1 = vec![5., 4., 3., 2., 1.];
        let x2 = vec![729.53, 439.0367, 42.054, 1., 0.];
        let x3 = vec![258.589, 616.297, 215.061, 498.361, 0.];
        let data = vec![("Y", y), ("X1", x1), ("X2", x2), ("X3", x3)];
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        let formula = "Y ~ X1 + X2 + X3";
        b.iter(|| {
            FormulaRegressionBuilder::new()
                .data(&data)
                .formula(formula)
                .fit_without_statistics()
        });
    }

    #[bench]
    fn bench_data_columns_without_stats(b: &mut Bencher) {
        let y = vec![1., 2., 3., 4., 5.];
        let x1 = vec![5., 4., 3., 2., 1.];
        let x2 = vec![729.53, 439.0367, 42.054, 1., 0.];
        let x3 = vec![258.589, 616.297, 215.061, 498.361, 0.];
        let data = vec![("Y", y), ("X1", x1), ("X2", x2), ("X3", x3)];
        let data = RegressionDataBuilder::new().build_from(data).unwrap();
        b.iter(|| {
            FormulaRegressionBuilder::new()
                .data(&data)
                .data_columns("Y", ["X1", "X2", "X3"])
                .fit_without_statistics()
        });
    }
}
