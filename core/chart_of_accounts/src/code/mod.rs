pub mod error;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use error::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash, Deserialize)]
pub struct AccountIdx(u64);
impl Display for AccountIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl From<u32> for AccountIdx {
    fn from(num: u32) -> Self {
        Self(num.into())
    }
}

impl AccountIdx {
    pub const FIRST: Self = Self(1);
    pub const MAX_TWO_DIGIT: Self = Self(99);
    pub const MAX_THREE_DIGIT: Self = Self(999);

    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartOfAccountCategoryCode {
    Assets = 1,
    Liabilities = 2,
    Equity = 3,
    Revenues = 4,
    Expenses = 5,
}

impl Display for ChartOfAccountCategoryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assets => write!(f, "Assets"),
            Self::Liabilities => write!(f, "Liabilities"),
            Self::Equity => write!(f, "Equity"),
            Self::Revenues => write!(f, "Revenues"),
            Self::Expenses => write!(f, "Expenses"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartOfAccountCode {
    Category(ChartOfAccountCategoryCode),
    ControlAccount {
        category: ChartOfAccountCategoryCode,
        index: AccountIdx,
    },
    ControlSubAccount {
        category: ChartOfAccountCategoryCode,
        control_index: AccountIdx,
        index: AccountIdx,
    },
    TransactionAccount {
        category: ChartOfAccountCategoryCode,
        control_index: AccountIdx,
        control_sub_index: AccountIdx,
        index: AccountIdx,
    },
}

impl std::fmt::Display for ChartOfAccountCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Category(category) => {
                write!(f, "{:01}000000", *category as u32)
            }
            Self::ControlAccount { category, index } => {
                write!(f, "{:01}{:02}0000", *category as u32, index)
            }
            Self::ControlSubAccount {
                category,
                control_index,
                index,
            } => {
                write!(
                    f,
                    "{:01}{:02}{:02}000",
                    *category as u32, control_index, index
                )
            }
            Self::TransactionAccount {
                category,
                control_index,
                control_sub_index,
                index,
            } => {
                write!(
                    f,
                    "{:01}{:02}{:02}{:03}",
                    *category as u32, control_index, control_sub_index, index
                )
            }
        }
    }
}

impl std::str::FromStr for ChartOfAccountCode {
    type Err = ChartOfAccountCodeError;

    fn from_str(s: &str) -> Result<Self, ChartOfAccountCodeError> {
        if s.len() != 8 {
            return Err(ChartOfAccountCodeError::InvalidCodeLength(s.to_string()));
        }

        fn parse_segment(s: &str) -> Result<u32, ChartOfAccountCodeError> {
            Ok(s.parse::<u32>()?)
        }

        let category_segment = parse_segment(&s[0..1])?;
        let category = Self::category_from_number(category_segment).ok_or(
            ChartOfAccountCodeError::InvalidCategoryNumber(category_segment),
        )?;

        let control = parse_segment(&s[1..3])?;
        let sub = parse_segment(&s[3..5])?;
        let trans = parse_segment(&s[5..8])?;

        match (control, sub, trans) {
            (0, 0, 0) => Ok(Self::Category(category)),
            (c, 0, 0) if c > 0 => Ok(Self::ControlAccount {
                category,
                index: c.into(),
            }),
            (c, s, 0) if c > 0 && s > 0 => Ok(Self::ControlSubAccount {
                category,
                control_index: c.into(),
                index: s.into(),
            }),
            (c, s, t) if c > 0 && s > 0 && t > 0 => Ok(Self::TransactionAccount {
                category,
                control_index: c.into(),
                control_sub_index: s.into(),
                index: t.into(),
            }),
            _ => Err(ChartOfAccountCodeError::InvalidCodeString(s.to_string())),
        }
    }
}

impl ChartOfAccountCode {
    fn category_from_number(num: u32) -> Option<ChartOfAccountCategoryCode> {
        match num {
            1 => Some(ChartOfAccountCategoryCode::Assets),
            2 => Some(ChartOfAccountCategoryCode::Liabilities),
            3 => Some(ChartOfAccountCategoryCode::Equity),
            4 => Some(ChartOfAccountCategoryCode::Revenues),
            5 => Some(ChartOfAccountCategoryCode::Expenses),
            _ => None,
        }
    }

    pub fn category(&self) -> ChartOfAccountCategoryCode {
        match *self {
            Self::Category(category) => category,
            Self::ControlAccount { category, .. } => category,
            Self::ControlSubAccount { category, .. } => category,
            Self::TransactionAccount { category, .. } => category,
        }
    }

    pub fn control_account(&self) -> Option<ChartOfAccountCode> {
        match *self {
            Self::ControlAccount { category, index } => {
                Some(Self::ControlAccount { category, index })
            }
            Self::ControlSubAccount {
                category,
                control_index,
                ..
            } => Some(Self::ControlAccount {
                category,
                index: control_index,
            }),
            Self::TransactionAccount {
                category,
                control_index,
                ..
            } => Some(Self::ControlAccount {
                category,
                index: control_index,
            }),
            Self::Category(_) => None,
        }
    }

    pub fn control_sub_account(&self) -> Option<ChartOfAccountCode> {
        match *self {
            Self::TransactionAccount {
                category,
                control_index,
                control_sub_index,
                ..
            } => Some(Self::ControlSubAccount {
                category,
                control_index,
                index: control_sub_index,
            }),
            Self::ControlSubAccount {
                category,
                control_index,
                index,
            } => Some(Self::ControlSubAccount {
                category,
                control_index,
                index,
            }),
            _ => None,
        }
    }

    pub const fn first_control_account(
        category: ChartOfAccountCode,
    ) -> Result<Self, ChartOfAccountCodeError> {
        match category {
            Self::Category(category) => Ok(Self::ControlAccount {
                category,
                index: AccountIdx::FIRST,
            }),
            _ => Err(ChartOfAccountCodeError::InvalidCategoryCodeForNewControlAccount),
        }
    }

    pub fn first_control_sub_account(
        control_account: &Self,
    ) -> Result<Self, ChartOfAccountCodeError> {
        match control_account {
            Self::ControlAccount { category, index } => Ok(Self::ControlSubAccount {
                category: *category,
                control_index: *index,
                index: AccountIdx::FIRST,
            }),
            _ => Err(ChartOfAccountCodeError::InvalidControlAccountCodeForNewControlSubAccount),
        }
    }

    pub fn first_transaction_account(
        control_sub_account: &Self,
    ) -> Result<Self, ChartOfAccountCodeError> {
        match control_sub_account {
            Self::ControlSubAccount {
                category,
                control_index,
                index,
            } => Ok(Self::TransactionAccount {
                category: *category,
                control_index: *control_index,
                control_sub_index: *index,
                index: AccountIdx::FIRST,
            }),
            _ => Err(ChartOfAccountCodeError::InvalidSubControlAccountCodeForNewTransactionAccount),
        }
    }

    pub fn next(&self) -> Result<Self, ChartOfAccountCodeError> {
        match *self {
            Self::Category(_) => Ok(*self), // Categories don't have next
            Self::ControlAccount { category, index } => {
                let next_index = index.next();
                if next_index > AccountIdx::MAX_TWO_DIGIT {
                    Err(ChartOfAccountCodeError::ControlIndexOverflowForCategory(
                        category,
                    ))
                } else {
                    Ok(Self::ControlAccount {
                        category,
                        index: next_index,
                    })
                }
            }
            Self::ControlSubAccount {
                category,
                control_index,
                index,
            } => {
                let next_index = index.next();
                if next_index > AccountIdx::MAX_TWO_DIGIT {
                    Err(
                        ChartOfAccountCodeError::ControlSubIndexOverflowForControlAccount(
                            category,
                            control_index,
                        ),
                    )
                } else {
                    Ok(Self::ControlSubAccount {
                        category,
                        control_index,
                        index: next_index,
                    })
                }
            }
            Self::TransactionAccount {
                category,
                control_index,
                control_sub_index,
                index,
            } => {
                let next_index = index.next();
                if next_index > AccountIdx::MAX_THREE_DIGIT {
                    Err(
                        ChartOfAccountCodeError::TransactionIndexOverflowForControlSubAccount(
                            category,
                            control_index,
                            control_sub_index,
                        ),
                    )
                } else {
                    Ok(Self::TransactionAccount {
                        category,
                        control_index,
                        control_sub_index,
                        index: next_index,
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    mod convert_to_string {
        use super::*;

        #[test]
        fn test_category_formatting() {
            let code = ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets);
            assert_eq!(code.to_string(), "1000000");
        }

        #[test]
        fn test_control_account_formatting() {
            let code = ChartOfAccountCode::ControlAccount {
                category: ChartOfAccountCategoryCode::Liabilities,
                index: 1.into(),
            };
            assert_eq!(code.to_string(), "2010000");
        }

        #[test]
        fn test_control_sub_account_formatting() {
            let code = ChartOfAccountCode::ControlSubAccount {
                category: ChartOfAccountCategoryCode::Equity,
                control_index: 1.into(),
                index: 2.into(),
            };
            assert_eq!(code.to_string(), "30102000");
        }

        #[test]
        fn test_transaction_account_formatting() {
            let code = ChartOfAccountCode::TransactionAccount {
                category: ChartOfAccountCategoryCode::Revenues,
                control_index: 1.into(),
                control_sub_index: 2.into(),
                index: 3.into(),
            };
            assert_eq!(code.to_string(), "40102003");
        }
    }

    mod parse_code_from_string {
        use super::*;

        #[test]
        fn test_parsing_valid_codes() {
            assert_eq!(
                ChartOfAccountCode::from_str("10000000").unwrap(),
                ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets)
            );

            assert_eq!(
                ChartOfAccountCode::from_str("20100000").unwrap(),
                ChartOfAccountCode::ControlAccount {
                    category: ChartOfAccountCategoryCode::Liabilities,
                    index: 1.into(),
                }
            );

            assert_eq!(
                ChartOfAccountCode::from_str("30102000").unwrap(),
                ChartOfAccountCode::ControlSubAccount {
                    category: ChartOfAccountCategoryCode::Equity,
                    control_index: 1.into(),
                    index: 2.into(),
                }
            );

            assert_eq!(
                ChartOfAccountCode::from_str("40102003").unwrap(),
                ChartOfAccountCode::TransactionAccount {
                    category: ChartOfAccountCategoryCode::Revenues,
                    control_index: 1.into(),
                    control_sub_index: 2.into(),
                    index: 3.into(),
                }
            );
        }

        #[test]
        fn test_invalid_code_length() {
            match ChartOfAccountCode::from_str("100") {
                Err(ChartOfAccountCodeError::InvalidCodeLength(code)) => {
                    assert_eq!(code, "100");
                }
                other => panic!("Expected InvalidCodeLength error, got {:?}", other),
            }
        }

        #[test]
        fn test_invalid_category() {
            match ChartOfAccountCode::from_str("90000000") {
                Err(ChartOfAccountCodeError::InvalidCategoryNumber(num)) => {
                    assert_eq!(num, 9);
                }
                other => panic!("Expected InvalidCategoryNumber error, got {:?}", other),
            }
        }

        #[test]
        fn test_invalid_code_format() {
            match ChartOfAccountCode::from_str("10002030") {
                Err(ChartOfAccountCodeError::InvalidCodeString(code)) => {
                    assert_eq!(code, "10002030");
                }
                other => panic!("Expected InvalidCodeString error, got {:?}", other),
            }
        }

        #[test]
        fn test_non_numeric_input() {
            match ChartOfAccountCode::from_str("A0000000") {
                Err(ChartOfAccountCodeError::ParseIntError(_)) => {
                    // ParseIntError doesn't implement PartialEq, so we just check the variant
                }
                other => panic!("Expected ParseIntError, got {:?}", other),
            }
        }
    }

    mod category_extraction_tests {
        use super::*;

        #[test]
        fn test_category_from_category_code() {
            for category in [
                ChartOfAccountCategoryCode::Assets,
                ChartOfAccountCategoryCode::Liabilities,
                ChartOfAccountCategoryCode::Equity,
                ChartOfAccountCategoryCode::Revenues,
                ChartOfAccountCategoryCode::Expenses,
            ] {
                let code = ChartOfAccountCode::Category(category);
                assert_eq!(code.category(), category);
            }
        }

        #[test]
        fn test_category_from_control_account() {
            for category in [
                ChartOfAccountCategoryCode::Assets,
                ChartOfAccountCategoryCode::Liabilities,
                ChartOfAccountCategoryCode::Equity,
                ChartOfAccountCategoryCode::Revenues,
                ChartOfAccountCategoryCode::Expenses,
            ] {
                let code = ChartOfAccountCode::ControlAccount {
                    category,
                    index: 1.into(),
                };
                assert_eq!(code.category(), category);
            }
        }

        #[test]
        fn test_category_from_control_sub_account() {
            for category in [
                ChartOfAccountCategoryCode::Assets,
                ChartOfAccountCategoryCode::Liabilities,
                ChartOfAccountCategoryCode::Equity,
                ChartOfAccountCategoryCode::Revenues,
                ChartOfAccountCategoryCode::Expenses,
            ] {
                let code = ChartOfAccountCode::ControlSubAccount {
                    category,
                    control_index: 1.into(),
                    index: 2.into(),
                };
                assert_eq!(code.category(), category);
            }
        }

        #[test]
        fn test_category_from_transaction_account() {
            for category in [
                ChartOfAccountCategoryCode::Assets,
                ChartOfAccountCategoryCode::Liabilities,
                ChartOfAccountCategoryCode::Equity,
                ChartOfAccountCategoryCode::Revenues,
                ChartOfAccountCategoryCode::Expenses,
            ] {
                let code = ChartOfAccountCode::TransactionAccount {
                    category,
                    control_index: 1.into(),
                    control_sub_index: 2.into(),
                    index: 3.into(),
                };
                assert_eq!(code.category(), category);
            }
        }
    }

    mod control_account_extraction_tests {
        use super::*;

        const CATEGORY: ChartOfAccountCategoryCode = ChartOfAccountCategoryCode::Assets;
        const CONTROL_INDEX: AccountIdx = AccountIdx::FIRST;
        const EXPECTED: ChartOfAccountCode = ChartOfAccountCode::ControlAccount {
            category: CATEGORY,
            index: CONTROL_INDEX,
        };

        #[test]
        fn test_control_account_from_transaction_account() {
            let transaction = ChartOfAccountCode::TransactionAccount {
                category: CATEGORY,
                control_index: CONTROL_INDEX,
                control_sub_index: 2.into(),
                index: 3.into(),
            };

            assert_eq!(transaction.control_account(), Some(EXPECTED));
        }

        #[test]
        fn test_control_account_from_control_sub_account() {
            let sub_account = ChartOfAccountCode::ControlSubAccount {
                category: CATEGORY,
                control_index: CONTROL_INDEX,
                index: 2.into(),
            };

            assert_eq!(sub_account.control_account(), Some(EXPECTED));
        }

        #[test]
        fn test_control_account_from_control_account() {
            let control_account = ChartOfAccountCode::ControlAccount {
                category: CATEGORY,
                index: CONTROL_INDEX,
            };

            assert_eq!(control_account.control_account(), Some(EXPECTED));
        }

        #[test]
        fn test_control_account_from_category_returns_none() {
            let category_code = ChartOfAccountCode::Category(CATEGORY);
            assert_eq!(category_code.control_account(), None);
        }
    }

    mod control_sub_account_extraction_tests {
        use super::*;

        const CATEGORY: ChartOfAccountCategoryCode = ChartOfAccountCategoryCode::Assets;
        const CONTROL_INDEX: AccountIdx = AccountIdx::FIRST;
        const SUB_INDEX: AccountIdx = AccountIdx::FIRST;
        const EXPECTED: ChartOfAccountCode = ChartOfAccountCode::ControlSubAccount {
            category: CATEGORY,
            control_index: CONTROL_INDEX,
            index: SUB_INDEX,
        };

        #[test]
        fn test_control_sub_account_from_transaction_account() {
            let transaction = ChartOfAccountCode::TransactionAccount {
                category: CATEGORY,
                control_index: CONTROL_INDEX,
                control_sub_index: SUB_INDEX,
                index: 3.into(),
            };

            assert_eq!(transaction.control_sub_account(), Some(EXPECTED));
        }

        #[test]
        fn test_control_sub_account_from_control_sub_account() {
            let sub_account = ChartOfAccountCode::ControlSubAccount {
                category: CATEGORY,
                control_index: CONTROL_INDEX,
                index: SUB_INDEX,
            };

            assert_eq!(sub_account.control_sub_account(), Some(EXPECTED));
        }

        #[test]
        fn test_control_sub_account_from_control_account_returns_none() {
            let control_account = ChartOfAccountCode::ControlAccount {
                category: CATEGORY,
                index: CONTROL_INDEX,
            };

            assert_eq!(control_account.control_sub_account(), None);
        }

        #[test]
        fn test_control_sub_account_from_category_returns_none() {
            let category_code = ChartOfAccountCode::Category(CATEGORY);
            assert_eq!(category_code.control_sub_account(), None);
        }
    }

    mod first_account_create {
        use super::*;

        #[test]
        fn test_first_control_account_creation() {
            let category = ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets);
            let control = ChartOfAccountCode::first_control_account(category).unwrap();

            assert_eq!(
                control,
                ChartOfAccountCode::ControlAccount {
                    category: ChartOfAccountCategoryCode::Assets,
                    index: AccountIdx::FIRST,
                }
            );
        }

        #[test]
        fn test_first_control_account_invalid_input() {
            let invalid_input = ChartOfAccountCode::ControlAccount {
                category: ChartOfAccountCategoryCode::Assets,
                index: 1.into(),
            };

            assert!(ChartOfAccountCode::first_control_account(invalid_input).is_err());
        }

        #[test]
        fn test_first_control_sub_account_creation() {
            let control = ChartOfAccountCode::ControlAccount {
                category: ChartOfAccountCategoryCode::Assets,
                index: AccountIdx::FIRST,
            };

            let sub = ChartOfAccountCode::first_control_sub_account(&control).unwrap();
            assert_eq!(
                sub,
                ChartOfAccountCode::ControlSubAccount {
                    category: ChartOfAccountCategoryCode::Assets,
                    control_index: AccountIdx::FIRST,
                    index: AccountIdx::FIRST,
                }
            );
        }

        #[test]
        fn test_first_control_sub_account_invalid_input() {
            let invalid_input = ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets);
            assert!(ChartOfAccountCode::first_control_sub_account(&invalid_input).is_err());
        }

        #[test]
        fn test_first_transaction_account_creation() {
            let sub = ChartOfAccountCode::ControlSubAccount {
                category: ChartOfAccountCategoryCode::Assets,
                control_index: AccountIdx::FIRST,
                index: AccountIdx::FIRST,
            };

            let transaction = ChartOfAccountCode::first_transaction_account(&sub).unwrap();
            assert_eq!(
                transaction,
                ChartOfAccountCode::TransactionAccount {
                    category: ChartOfAccountCategoryCode::Assets,
                    control_index: AccountIdx::FIRST,
                    control_sub_index: AccountIdx::FIRST,
                    index: AccountIdx::FIRST,
                }
            );
        }

        #[test]
        fn test_first_transaction_account_invalid_input() {
            let invalid_input = ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets);
            assert!(ChartOfAccountCode::first_transaction_account(&invalid_input).is_err());
        }
    }

    mod next_account_create {
        use super::*;

        #[test]
        fn test_next_control_account_success() {
            let control = ChartOfAccountCode::ControlAccount {
                category: ChartOfAccountCategoryCode::Assets,
                index: 1.into(),
            };

            let next_control = control.next().unwrap();
            assert_eq!(
                next_control,
                ChartOfAccountCode::ControlAccount {
                    category: ChartOfAccountCategoryCode::Assets,
                    index: 2.into(),
                }
            );
        }

        #[test]
        fn test_next_control_account_overflow() {
            let max_control = ChartOfAccountCode::ControlAccount {
                category: ChartOfAccountCategoryCode::Assets,
                index: AccountIdx::MAX_TWO_DIGIT,
            };
            assert!(max_control.next().is_err());
        }

        #[test]
        fn test_next_control_sub_account_success() {
            let sub = ChartOfAccountCode::ControlSubAccount {
                category: ChartOfAccountCategoryCode::Assets,
                control_index: 1.into(),
                index: 1.into(),
            };

            let next_sub = sub.next().unwrap();
            assert_eq!(
                next_sub,
                ChartOfAccountCode::ControlSubAccount {
                    category: ChartOfAccountCategoryCode::Assets,
                    control_index: 1.into(),
                    index: 2.into(),
                }
            );
        }

        #[test]
        fn test_next_control_sub_account_overflow() {
            let max_sub = ChartOfAccountCode::ControlSubAccount {
                category: ChartOfAccountCategoryCode::Assets,
                control_index: 1.into(),
                index: AccountIdx::MAX_TWO_DIGIT,
            };
            assert!(max_sub.next().is_err());
        }

        #[test]
        fn test_next_transaction_account_success() {
            let transaction = ChartOfAccountCode::TransactionAccount {
                category: ChartOfAccountCategoryCode::Assets,
                control_index: 1.into(),
                control_sub_index: 1.into(),
                index: 1.into(),
            };

            let next_transaction = transaction.next().unwrap();
            assert_eq!(
                next_transaction,
                ChartOfAccountCode::TransactionAccount {
                    category: ChartOfAccountCategoryCode::Assets,
                    control_index: 1.into(),
                    control_sub_index: 1.into(),
                    index: 2.into(),
                }
            );
        }

        #[test]
        fn test_next_transaction_account_overflow() {
            let max_transaction = ChartOfAccountCode::TransactionAccount {
                category: ChartOfAccountCategoryCode::Assets,
                control_index: 1.into(),
                control_sub_index: 1.into(),
                index: AccountIdx::MAX_THREE_DIGIT,
            };
            assert!(max_transaction.next().is_err());
        }

        #[test]
        fn test_next_category_returns_same() {
            let category = ChartOfAccountCode::Category(ChartOfAccountCategoryCode::Assets);
            let next_category = category.next().unwrap();
            assert_eq!(category, next_category);
        }
    }
}
