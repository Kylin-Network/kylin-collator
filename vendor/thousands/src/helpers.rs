use super::SeparatorPolicy;

#[derive(Debug)]
pub struct SeparatorIterator<'a> {
    groups:                  &'a [u8],
    repeat_groups_remaining: usize,
    current_group_index:     usize,
    current_group_size:      usize,
    len:                     usize,
}

impl<'a> SeparatorIterator<'a> {
    pub fn new(policy: &'a SeparatorPolicy, len: usize) -> Self {
        let groups = &policy.groups;

        let mut sum = 0;

        for (index, &group) in groups.into_iter().enumerate() {
            sum += group as usize;

            if len <= sum {
                return SeparatorIterator {
                    groups,
                    repeat_groups_remaining: 0,
                    current_group_index:     index,
                    current_group_size:      len - (sum - group as usize),
                    len,
                }
            }
        }

        let repeat_group_len = match groups.last() {
            Some(n) => *n as usize,
            None    =>
                return SeparatorIterator {
                    groups:                  &[],
                    repeat_groups_remaining: 0,
                    current_group_index:     0,
                    current_group_size:      0,
                    len,
                }
        };

        let len_remaining = len - sum;
        let (repeat_groups_remaining, current_group_size)
                          = ceil_div_mod(len_remaining, repeat_group_len);

        SeparatorIterator {
            groups,
            repeat_groups_remaining,
            current_group_index: groups.len() - 1,
            current_group_size,
            len,
        }
    }

    /// How many separators remain?
    pub fn sep_len(&self) -> usize {
        self.current_group_index + self.repeat_groups_remaining
    }
}

impl<'a> Iterator for SeparatorIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.len = self.len.checked_sub(1)?;

        self.current_group_size = self.current_group_size.saturating_sub(1);
        if self.current_group_size > 0 {
            return Some(false);
        }

        if let Some(repeat_groups_remaining) = self.repeat_groups_remaining.checked_sub(1) {
            self.repeat_groups_remaining = repeat_groups_remaining;
        } else if let Some(current_group_index) = self.current_group_index.checked_sub(1) {
            self.current_group_index = current_group_index;
        } else {
            return Some(false);
        }

        self.current_group_size = self.groups[self.current_group_index] as usize;
        return Some(true);
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> ExactSizeIterator for SeparatorIterator<'a> {
    fn len(&self) -> usize {
        self.len
    }
}

fn ceil_div_mod(n: usize, m: usize) -> (usize, usize) {
    let round_up = n + m - 1;
    (round_up / m, round_up % m + 1)
}

#[cfg(test)]
mod test_common {
    use super::super::*;
    pub use super::*;

    pub fn make_policy(groups: &[u8]) -> SeparatorPolicy {
        let mut result = policies::COMMA_SEPARATOR;
        result.groups = groups;
        result
    }
}

#[cfg(test)]
mod grouping_test {
    use super::test_common::*;

    fn group_string(groups: &[u8], digits: &str) -> String {
        use std::iter::once;

        let policy = &make_policy(groups);
        let iter = SeparatorIterator::new(policy, digits.chars().count());

        digits.chars().zip(iter)
            .flat_map(|(digit, comma_after)|
                    once(digit)
                        .chain(if comma_after { Some(',') } else { None }))
            .collect()
    }

    macro_rules! grouping_test {
        ( $name:ident, $groups:tt, $result:tt ) => {
            #[test]
            fn $name() {
                let result = $result;
                let input = $result.chars().filter(|&c| c != ',').collect::<String>();
                assert_eq!(group_string(&$groups, &input), result);
            }
        };
    }

    grouping_test!(by_nothing_of_0, [], "");
    grouping_test!(by_nothing_of_1, [], "1");
    grouping_test!(by_nothing_of_2, [], "21");
    grouping_test!(by_nothing_of_3, [], "321");

    grouping_test!(by_1s_of_0, [1], "");
    grouping_test!(by_1s_of_1, [1], "1");
    grouping_test!(by_1s_of_2, [1], "2,1");
    grouping_test!(by_1s_of_3, [1], "3,2,1");

    grouping_test!(by_2s_of_0, [2], "");
    grouping_test!(by_2s_of_1, [2], "1");
    grouping_test!(by_2s_of_2, [2], "21");
    grouping_test!(by_2s_of_3, [2], "3,21");
    grouping_test!(by_2s_of_4, [2], "43,21");
    grouping_test!(by_2s_of_5, [2], "5,43,21");
    grouping_test!(by_2s_of_6, [2], "65,43,21");
    grouping_test!(by_2s_of_7, [2], "7,65,43,21");
    grouping_test!(by_2s_of_8, [2], "87,65,43,21");
    grouping_test!(by_2s_of_9, [2], "9,87,65,43,21");

    grouping_test!(by_3s_of_1, [3], "1");
    grouping_test!(by_3s_of_2, [3], "21");
    grouping_test!(by_3s_of_3, [3], "321");
    grouping_test!(by_3s_of_4, [3], "4,321");
    grouping_test!(by_3s_of_5, [3], "54,321");
    grouping_test!(by_3s_of_6, [3], "654,321");
    grouping_test!(by_3s_of_7, [3], "7,654,321");
    grouping_test!(by_3s_of_8, [3], "87,654,321");
    grouping_test!(by_3s_of_9, [3], "987,654,321");

    grouping_test!(by_2s3_of_1, [3, 2], "1");
    grouping_test!(by_2s3_of_2, [3, 2], "21");
    grouping_test!(by_2s3_of_3, [3, 2], "321");
    grouping_test!(by_2s3_of_4, [3, 2], "4,321");
    grouping_test!(by_2s3_of_5, [3, 2], "54,321");
    grouping_test!(by_2s3_of_6, [3, 2], "6,54,321");
    grouping_test!(by_2s3_of_7, [3, 2], "76,54,321");
    grouping_test!(by_2s3_of_8, [3, 2], "8,76,54,321");
    grouping_test!(by_2s3_of_9, [3, 2], "98,76,54,321");

    grouping_test!(by_5s4321_of_20, [1, 2, 3, 4, 5],
                   "KJIHG,FEDCB,A987,654,32,1");
    grouping_test!(by_5s4321_of_16, [1, 2, 3, 4, 5],
                   "G,FEDCB,A987,654,32,1");
    grouping_test!(by_5s4321_of_11, [1, 2, 3, 4, 5],
                   "B,A987,654,32,1");
    grouping_test!(by_5s4321_of_10, [1, 2, 3, 4, 5],
                   "A987,654,32,1");
    grouping_test!(by_5s4321_of_9, [1, 2, 3, 4, 5],
                   "987,654,32,1");
    grouping_test!(by_5s4321_of_7, [1, 2, 3, 4, 5],
                   "7,654,32,1");
    grouping_test!(by_5s4321_of_1, [1, 2, 3, 4, 5],
                   "1");
    grouping_test!(by_5s4321_of_0, [1, 2, 3, 4, 5],
                   "");
}

#[cfg(test)]
mod sep_len_test {
    use super::test_common::*;

    fn run_iterator(mut iter: SeparatorIterator) -> (Vec<usize>, Vec<usize>) {
        let mut predictions = Vec::with_capacity(iter.len());
        let mut actuals     = Vec::with_capacity(iter.len());

        let mut prediction;
        while let Some(actual) = {
            prediction = iter.sep_len();
            iter.next()
        } {
            predictions.push(prediction);
            actuals.push(if actual { 1 } else { 0 })
        }

        let mut acc = 0;
        for actual in actuals.iter_mut().rev() {
            acc += *actual;
            *actual = acc;
        }

        (predictions, actuals)
    }

    macro_rules! run_down {
        ( $name:ident, $groups:tt, $size:tt ) => {
            #[test]
            fn $name() {
                let policy = &make_policy(&$groups);

                let (predictions, actuals) =
                        run_iterator(SeparatorIterator::new(policy, $size));

                assert_eq!(predictions, actuals);
            }
        };
    }

    run_down!(by_nothing_of_10, [], 10);

    run_down!(by_3s_of_10, [3], 10);

    run_down!(by_2s_of_10, [2], 10);
    run_down!(by_2s_of_9, [2], 9);
    run_down!(by_2s_of_8, [2], 8);
    run_down!(by_2s_of_7, [2], 7);
    run_down!(by_2s_of_6, [2], 6);
    run_down!(by_2s_of_5, [2], 5);
    run_down!(by_2s_of_4, [2], 4);
    run_down!(by_2s_of_3, [2], 3);
    run_down!(by_2s_of_2, [2], 2);
    run_down!(by_2s_of_1, [2], 1);
    run_down!(by_2s_of_0, [2], 0);

    run_down!(by_1s23_of_10, [3, 2, 1], 10);
    run_down!(by_1s23_of_9, [3, 2, 1], 9);
    run_down!(by_1s23_of_8, [3, 2, 1], 8);
    run_down!(by_1s23_of_7, [3, 2, 1], 7);
    run_down!(by_1s23_of_6, [3, 2, 1], 6);
    run_down!(by_1s23_of_5, [3, 2, 1], 5);
    run_down!(by_1s23_of_4, [3, 2, 1], 4);
    run_down!(by_1s23_of_3, [3, 2, 1], 3);
    run_down!(by_1s23_of_2, [3, 2, 1], 2);
    run_down!(by_1s23_of_1, [3, 2, 1], 1);
    run_down!(by_1s23_of_0, [3, 2, 1], 0);
}
