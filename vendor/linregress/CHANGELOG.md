## 0.4.4
- Add `data_columns` method to `FormulaRegressionBuilder`.
  It allows setting the regressand a regressors without using a formula string.
- Add `fit_low_level_regression_model` and `fit_low_level_regression_model_without_statistics`
  functions for performing a regression directly on a matrix of input data.

## 0.4.3
- Update `statrs` dependency to `0.15.0` to avoid multiple versions of `nalgebra` in out dependency tree

## 0.4.2
- Update `nalgebra` to `0.27.1` in response to RUSTSEC-2021-0070
- Update `statrs` to `0.14.0`
