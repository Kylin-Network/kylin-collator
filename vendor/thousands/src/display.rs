use std::fmt::Display;

use super::{Separable, SeparatorPolicy};
use super::helpers::SeparatorIterator;

impl Separable for str {
    fn separate_by_policy(&self, policy: SeparatorPolicy) -> String {
        let (before, number, after, count) = find_span(&self, |c| policy.digits.contains(&c));
        let iter = SeparatorIterator::new(&policy, count);

        let mut result = String::with_capacity(self.len() + iter.sep_len());

        result.push_str(before);

        for (digit, comma_after) in number.chars().zip(iter) {
            result.push(digit);
            if comma_after {
                result.push_str(policy.separator);
            }
        }

        result.push_str(after);

        result
    }
}

impl<T: Display> Separable for T {
    fn separate_by_policy(&self, policy: SeparatorPolicy) -> String {
        self.to_string().as_str().separate_by_policy(policy)
    }
}

fn find_span<F: Fn(char) -> bool>(s: &str, is_digit: F) -> (&str, &str, &str, usize) {
    let start        = len_not_matching(s, &is_digit);
    let (len, count) = len_and_count_matching(&s[start ..], &is_digit);
    let limit        = start + len;

    (&s[.. start], &s[start .. limit], &s[limit ..], count)
}

fn len_not_matching<F>(s: &str, mut pred: F) -> usize
where F: FnMut(char) -> bool {

    if let Some((i, _)) = s.char_indices().find(|p| pred(p.1)) {
        i
    } else {
        s.len()
    }
}

fn len_and_count_matching<F>(s: &str, pred: F) -> (usize, usize)
where F: Fn(char) -> bool {

    let mut count = 0;
    let     len = len_not_matching(s, |c|
        if pred(c) {
            count += 1;
            false
        } else {
            true
        });

    (len, count)
}

#[cfg(test)]
mod test {
    use super::super::{Separable, SeparatorPolicy, digits, policies};

    #[test]
    fn integer_thousands_commas() {
        assert_eq!( "12345".separate_with_commas(),
                    "12,345" );
    }

    #[test]
    fn smilies() {
        let policy = SeparatorPolicy {
            separator: "😃😃",
            groups:    &[1],
            digits:    &['🙁'],
        };

        assert_eq!( "  🙁🙁🙁🙁🙁  ".separate_by_policy(policy),
                    "  🙁😃😃🙁😃😃🙁😃😃🙁😃😃🙁  " );
    }

    #[test]
    fn three_two_two_two() {
        let policy = SeparatorPolicy {
            separator: ",",
            groups:    &[3, 2],
            digits:    &digits::ASCII_DECIMAL,
        };

        assert_eq!( "1234567890".separate_by_policy(policy),
                    "1,23,45,67,890" );
    }

    #[test]
    fn minus_sign_and_decimal_point() {
        assert_eq!( "-1234.5".separate_with_commas(),
                    "-1,234.5" );
    }

    #[test]
    fn hex_four() {
        assert_eq!( "deadbeef".separate_by_policy(policies::HEX_FOUR),
                    "dead beef" );
    }
}
