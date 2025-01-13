pub mod error;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::primitives::{ChartId, DebitOrCredit};
use error::*;

const ENCODED_PATH_WIDTH: usize = 8;

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
pub enum ChartCategoryPath {
    Assets,
    Liabilities,
    Equity,
    Revenues,
    Expenses,
}

impl std::fmt::Display for ChartCategoryPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0<ENCODED_PATH_WIDTH$}", self.index())
    }
}

impl ChartCategoryPath {
    fn index(&self) -> AccountIdx {
        match self {
            Self::Assets => AccountIdx::from(1),
            Self::Liabilities => AccountIdx::from(2),
            Self::Equity => AccountIdx::from(3),
            Self::Revenues => AccountIdx::from(4),
            Self::Expenses => AccountIdx::from(5),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartPath {
    ControlAccount {
        category: ChartCategoryPath,
        index: AccountIdx,
    },
    ControlSubAccount {
        category: ChartCategoryPath,
        control_index: AccountIdx,
        index: AccountIdx,
    },
    TransactionAccount {
        category: ChartCategoryPath,
        control_index: AccountIdx,
        control_sub_index: AccountIdx,
        index: AccountIdx,
    },
}

impl std::fmt::Display for ChartPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ControlAccount { category, index } => {
                write!(
                    f,
                    "{:0<ENCODED_PATH_WIDTH$}",
                    format!("{:01}{:02}", category.index(), index)
                )
            }
            Self::ControlSubAccount {
                category,
                control_index,
                index,
            } => {
                write!(
                    f,
                    "{:0<ENCODED_PATH_WIDTH$}",
                    format!("{:01}{:02}{:02}", category.index(), control_index, index)
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
                    "{:0<ENCODED_PATH_WIDTH$}",
                    format!(
                        "{:01}{:02}{:02}{:03}",
                        category.index(),
                        control_index,
                        control_sub_index,
                        index
                    )
                )
            }
        }
    }
}

impl ChartPath {
    pub fn normal_balance_type(&self) -> DebitOrCredit {
        match self.category() {
            ChartCategoryPath::Assets | ChartCategoryPath::Expenses => DebitOrCredit::Debit,
            _ => DebitOrCredit::Credit,
        }
    }

    pub fn path_encode(&self, chart_id: ChartId) -> String {
        format!("{}::{}", chart_id, self)
    }

    pub fn category(&self) -> ChartCategoryPath {
        match *self {
            Self::ControlAccount { category, .. } => category,
            Self::ControlSubAccount { category, .. } => category,
            Self::TransactionAccount { category, .. } => category,
        }
    }

    pub fn control_account(&self) -> ChartPath {
        match *self {
            Self::ControlAccount { category, index } => Self::ControlAccount { category, index },
            Self::ControlSubAccount {
                category,
                control_index,
                ..
            } => Self::ControlAccount {
                category,
                index: control_index,
            },
            Self::TransactionAccount {
                category,
                control_index,
                ..
            } => Self::ControlAccount {
                category,
                index: control_index,
            },
        }
    }

    pub fn control_sub_account(&self) -> Option<ChartPath> {
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

    pub const fn first_control_account(category: ChartCategoryPath) -> Self {
        Self::ControlAccount {
            category,
            index: AccountIdx::FIRST,
        }
    }

    pub fn first_control_sub_account(control_account: &Self) -> Result<Self, ChartPathError> {
        match control_account {
            Self::ControlAccount { category, index } => Ok(Self::ControlSubAccount {
                category: *category,
                control_index: *index,
                index: AccountIdx::FIRST,
            }),
            _ => Err(ChartPathError::InvalidControlAccountPathForNewControlSubAccount),
        }
    }

    pub fn first_transaction_account(control_sub_account: &Self) -> Result<Self, ChartPathError> {
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
            _ => Err(ChartPathError::InvalidSubControlAccountPathForNewTransactionAccount),
        }
    }

    pub fn next(&self) -> Result<Self, ChartPathError> {
        match *self {
            Self::ControlAccount { category, index } => {
                let next_index = index.next();
                if next_index > AccountIdx::MAX_TWO_DIGIT {
                    Err(ChartPathError::ControlIndexOverflowForCategory(category))
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
                    Err(ChartPathError::ControlSubIndexOverflowForControlAccount(
                        category,
                        control_index,
                    ))
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
                        ChartPathError::TransactionIndexOverflowForControlSubAccount(
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

    mod convert_to_string {
        use super::*;

        #[test]
        fn test_category_formatting() {
            let code = ChartCategoryPath::Assets;
            assert_eq!(code.to_string(), "10000000");
        }

        #[test]
        fn test_control_account_formatting() {
            let code = ChartPath::ControlAccount {
                category: ChartCategoryPath::Liabilities,
                index: 1.into(),
            };
            assert_eq!(code.to_string(), "20100000");
        }

        #[test]
        fn test_control_sub_account_formatting() {
            let code = ChartPath::ControlSubAccount {
                category: ChartCategoryPath::Equity,
                control_index: 1.into(),
                index: 2.into(),
            };
            assert_eq!(code.to_string(), "30102000");
        }

        #[test]
        fn test_transaction_account_formatting() {
            let code = ChartPath::TransactionAccount {
                category: ChartCategoryPath::Revenues,
                control_index: 1.into(),
                control_sub_index: 2.into(),
                index: 3.into(),
            };
            assert_eq!(code.to_string(), "40102003");
        }
    }

    mod category_extraction_tests {
        use super::*;

        #[test]
        fn test_category_from_control_account() {
            for category in [
                ChartCategoryPath::Assets,
                ChartCategoryPath::Liabilities,
                ChartCategoryPath::Equity,
                ChartCategoryPath::Revenues,
                ChartCategoryPath::Expenses,
            ] {
                let code = ChartPath::ControlAccount {
                    category,
                    index: 1.into(),
                };
                assert_eq!(code.category(), category);
            }
        }

        #[test]
        fn test_category_from_control_sub_account() {
            for category in [
                ChartCategoryPath::Assets,
                ChartCategoryPath::Liabilities,
                ChartCategoryPath::Equity,
                ChartCategoryPath::Revenues,
                ChartCategoryPath::Expenses,
            ] {
                let code = ChartPath::ControlSubAccount {
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
                ChartCategoryPath::Assets,
                ChartCategoryPath::Liabilities,
                ChartCategoryPath::Equity,
                ChartCategoryPath::Revenues,
                ChartCategoryPath::Expenses,
            ] {
                let code = ChartPath::TransactionAccount {
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

        const CATEGORY: ChartCategoryPath = ChartCategoryPath::Assets;
        const CONTROL_INDEX: AccountIdx = AccountIdx::FIRST;
        const EXPECTED: ChartPath = ChartPath::ControlAccount {
            category: CATEGORY,
            index: CONTROL_INDEX,
        };

        #[test]
        fn test_control_account_from_transaction_account() {
            let transaction = ChartPath::TransactionAccount {
                category: CATEGORY,
                control_index: CONTROL_INDEX,
                control_sub_index: 2.into(),
                index: 3.into(),
            };

            assert_eq!(transaction.control_account(), EXPECTED);
        }

        #[test]
        fn test_control_account_from_control_sub_account() {
            let sub_account = ChartPath::ControlSubAccount {
                category: CATEGORY,
                control_index: CONTROL_INDEX,
                index: 2.into(),
            };

            assert_eq!(sub_account.control_account(), EXPECTED);
        }

        #[test]
        fn test_control_account_from_control_account() {
            let control_account = ChartPath::ControlAccount {
                category: CATEGORY,
                index: CONTROL_INDEX,
            };

            assert_eq!(control_account.control_account(), EXPECTED);
        }
    }

    mod control_sub_account_extraction_tests {
        use super::*;

        const CATEGORY: ChartCategoryPath = ChartCategoryPath::Assets;
        const CONTROL_INDEX: AccountIdx = AccountIdx::FIRST;
        const SUB_INDEX: AccountIdx = AccountIdx::FIRST;
        const EXPECTED: ChartPath = ChartPath::ControlSubAccount {
            category: CATEGORY,
            control_index: CONTROL_INDEX,
            index: SUB_INDEX,
        };

        #[test]
        fn test_control_sub_account_from_transaction_account() {
            let transaction = ChartPath::TransactionAccount {
                category: CATEGORY,
                control_index: CONTROL_INDEX,
                control_sub_index: SUB_INDEX,
                index: 3.into(),
            };

            assert_eq!(transaction.control_sub_account(), Some(EXPECTED));
        }

        #[test]
        fn test_control_sub_account_from_control_sub_account() {
            let sub_account = ChartPath::ControlSubAccount {
                category: CATEGORY,
                control_index: CONTROL_INDEX,
                index: SUB_INDEX,
            };

            assert_eq!(sub_account.control_sub_account(), Some(EXPECTED));
        }

        #[test]
        fn test_control_sub_account_from_control_account_returns_none() {
            let control_account = ChartPath::ControlAccount {
                category: CATEGORY,
                index: CONTROL_INDEX,
            };

            assert_eq!(control_account.control_sub_account(), None);
        }
    }

    mod first_account_create {
        use super::*;

        #[test]
        fn test_first_control_account_creation() {
            let category = ChartCategoryPath::Assets;
            let control = ChartPath::first_control_account(category);

            assert_eq!(
                control,
                ChartPath::ControlAccount {
                    category: ChartCategoryPath::Assets,
                    index: AccountIdx::FIRST,
                }
            );
        }

        #[test]
        fn test_first_control_sub_account_creation() {
            let control = ChartPath::ControlAccount {
                category: ChartCategoryPath::Assets,
                index: AccountIdx::FIRST,
            };

            let sub = ChartPath::first_control_sub_account(&control).unwrap();
            assert_eq!(
                sub,
                ChartPath::ControlSubAccount {
                    category: ChartCategoryPath::Assets,
                    control_index: AccountIdx::FIRST,
                    index: AccountIdx::FIRST,
                }
            );
        }

        #[test]
        fn test_first_control_sub_account_invalid_input() {
            let invalid_input = ChartPath::ControlSubAccount {
                category: ChartCategoryPath::Assets,
                control_index: 1.into(),
                index: 1.into(),
            };
            assert!(ChartPath::first_control_sub_account(&invalid_input).is_err());
        }

        #[test]
        fn test_first_transaction_account_creation() {
            let sub = ChartPath::ControlSubAccount {
                category: ChartCategoryPath::Assets,
                control_index: AccountIdx::FIRST,
                index: AccountIdx::FIRST,
            };

            let transaction = ChartPath::first_transaction_account(&sub).unwrap();
            assert_eq!(
                transaction,
                ChartPath::TransactionAccount {
                    category: ChartCategoryPath::Assets,
                    control_index: AccountIdx::FIRST,
                    control_sub_index: AccountIdx::FIRST,
                    index: AccountIdx::FIRST,
                }
            );
        }

        #[test]
        fn test_first_transaction_account_invalid_input() {
            let invalid_input = ChartPath::ControlAccount {
                category: ChartCategoryPath::Assets,
                index: 1.into(),
            };
            assert!(ChartPath::first_transaction_account(&invalid_input).is_err());
        }
    }

    mod next_account_create {
        use super::*;

        #[test]
        fn test_next_control_account_success() {
            let control = ChartPath::ControlAccount {
                category: ChartCategoryPath::Assets,
                index: 1.into(),
            };

            let next_control = control.next().unwrap();
            assert_eq!(
                next_control,
                ChartPath::ControlAccount {
                    category: ChartCategoryPath::Assets,
                    index: 2.into(),
                }
            );
        }

        #[test]
        fn test_next_control_account_overflow() {
            let max_control = ChartPath::ControlAccount {
                category: ChartCategoryPath::Assets,
                index: AccountIdx::MAX_TWO_DIGIT,
            };
            assert!(max_control.next().is_err());
        }

        #[test]
        fn test_next_control_sub_account_success() {
            let sub = ChartPath::ControlSubAccount {
                category: ChartCategoryPath::Assets,
                control_index: 1.into(),
                index: 1.into(),
            };

            let next_sub = sub.next().unwrap();
            assert_eq!(
                next_sub,
                ChartPath::ControlSubAccount {
                    category: ChartCategoryPath::Assets,
                    control_index: 1.into(),
                    index: 2.into(),
                }
            );
        }

        #[test]
        fn test_next_control_sub_account_overflow() {
            let max_sub = ChartPath::ControlSubAccount {
                category: ChartCategoryPath::Assets,
                control_index: 1.into(),
                index: AccountIdx::MAX_TWO_DIGIT,
            };
            assert!(max_sub.next().is_err());
        }

        #[test]
        fn test_next_transaction_account_success() {
            let transaction = ChartPath::TransactionAccount {
                category: ChartCategoryPath::Assets,
                control_index: 1.into(),
                control_sub_index: 1.into(),
                index: 1.into(),
            };

            let next_transaction = transaction.next().unwrap();
            assert_eq!(
                next_transaction,
                ChartPath::TransactionAccount {
                    category: ChartCategoryPath::Assets,
                    control_index: 1.into(),
                    control_sub_index: 1.into(),
                    index: 2.into(),
                }
            );
        }

        #[test]
        fn test_next_transaction_account_overflow() {
            let max_transaction = ChartPath::TransactionAccount {
                category: ChartCategoryPath::Assets,
                control_index: 1.into(),
                control_sub_index: 1.into(),
                index: AccountIdx::MAX_THREE_DIGIT,
            };
            assert!(max_transaction.next().is_err());
        }
    }
}
