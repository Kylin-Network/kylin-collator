use super::{SeparatorPolicy, policies};

/// Provides methods for formatting numbers with separators between the digits.
pub trait Separable {
    /// Inserts a comma every three digits from the right.
    ///
    /// This is equivalent to `self.separate_by_policy(policies::COMMA_SEPARATOR)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use thousands::*;
    /// assert_eq!( 12345.separate_with_commas(), "12,345" );
    /// ```
    fn separate_with_commas(&self) -> String {
        self.separate_by_policy(policies::COMMA_SEPARATOR)
    }

    /// Inserts a space every three digits from the right.
    ///
    /// This is equivalent to `self.separate_by_policy(policies::SPACE_SEPARATOR)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use thousands::*;
    /// assert_eq!( 12345.separate_with_spaces(), "12 345" );
    /// ```
    fn separate_with_spaces(&self) -> String {
        self.separate_by_policy(policies::SPACE_SEPARATOR)
    }

    /// Inserts a period every three digits from the right.
    ///
    /// This is equivalent to `self.separate_by_policy(policies::DOT_SEPARATOR)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use thousands::*;
    /// assert_eq!( 12345.separate_with_dots(), "12.345" );
    /// ```
    fn separate_with_dots(&self) -> String {
        self.separate_by_policy(policies::DOT_SEPARATOR)
    }

    /// Inserts an underscore every three digits from the right.
    ///
    /// This is equivalent to `self.separate_by_policy(policies::UNDERSCORE_SEPARATOR)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use thousands::*;
    /// assert_eq!( 12345.separate_with_underscores(), "12_345" );
    /// ```
    fn separate_with_underscores(&self) -> String {
        self.separate_by_policy(policies::UNDERSCORE_SEPARATOR)
    }

    /// Adds separators according to the given [`SeparatorPolicy`].
    ///
    /// # Examples
    ///
    /// ```
    /// use thousands::{Separable, SeparatorPolicy, digits};
    ///
    /// let policy = SeparatorPolicy {
    ///     separator:  ":",
    ///     groups:     &[1, 2, 3, 4],
    ///     digits:     digits::ASCII_DECIMAL,
    /// };
    ///
    /// assert_eq!( 1234567654321u64.separate_by_policy(policy),
    ///             "123:4567:654:32:1" );
    /// ```
    ///
    /// [`SeparatorPolicy`]: struct.SeparatorPolicy.html
    fn separate_by_policy(&self, policy: SeparatorPolicy) -> String;
}

