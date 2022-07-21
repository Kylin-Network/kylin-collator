use super::digits::*;

/// A policy for inserting separators into numbers.
///
/// The configurable aspects are:
///
///   - The separator character to insert.
///
///   - How to group the separators.
///
///   - What characters are considered digits (for skipping non-digits such as
///     a minus sign).
#[derive(Debug, Clone, Copy)]
pub struct SeparatorPolicy<'a> {
    /// The separator to insert.
    pub separator: &'a str,
    /// The grouping. The numbers in this array give the size of the groups, from
    /// right to left, with the last number in the array giving the size of all
    /// subsequent groups.
    ///
    /// So to group by threes, as is typical in many places,
    /// this array should be `&[3]`. However, to get a grouping like `1,23,45,678`,
    /// where the last group has size three and the others size two, you would use
    /// `&[3, 2]`.
    pub groups:    &'a [u8],
    /// The characters that are considered digits. If there are multiple groups of
    /// digits separated by non-digits, we only add separators to the first group.
    /// This means, for example, that the number `-12345.67` will only have separators
    /// inserted into the `12345` portion.
    pub digits:    &'a [char],
}

/// Policy for placing a comma every three decimal digits.
pub const COMMA_SEPARATOR: SeparatorPolicy = SeparatorPolicy {
    separator:  ",",
    groups:     &[3],
    digits:     ASCII_DECIMAL,
};

/// Policy for placing a space every three decimal digits.
pub const SPACE_SEPARATOR: SeparatorPolicy = SeparatorPolicy {
    separator:  " ",
    groups:     &[3],
    digits:     ASCII_DECIMAL,
};

/// Policy for placing a period every three decimal digits.
pub const DOT_SEPARATOR: SeparatorPolicy = SeparatorPolicy {
    separator:  ".",
    groups:     &[3],
    digits:     ASCII_DECIMAL,
};

/// Policy for placing an underscore every three decimal digits.
pub const UNDERSCORE_SEPARATOR: SeparatorPolicy = SeparatorPolicy {
    separator:  "_",
    groups:     &[3],
    digits:     ASCII_DECIMAL,
};

/// Policy for placing a space every four hexadecimal digits.
pub const HEX_FOUR: SeparatorPolicy = SeparatorPolicy {
    separator:  " ",
    groups:     &[4],
    digits: ASCII_HEXADECIMAL,
};
