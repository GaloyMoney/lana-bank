pub mod error;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::primitives::{ChartId, DebitOrCredit};
use error::*;

const ENCODED_PATH_WIDTH: usize = 5;

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

    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChartCategory {
    Assets,
    Liabilities,
    Equity,
    Revenues,
    Expenses,
}

impl std::fmt::Display for ChartCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0<ENCODED_PATH_WIDTH$}", self.index())
    }
}

impl ChartCategory {
    fn index(&self) -> AccountIdx {
        match self {
            Self::Assets => AccountIdx::from(1),
            Self::Liabilities => AccountIdx::from(2),
            Self::Equity => AccountIdx::from(3),
            Self::Revenues => AccountIdx::from(4),
            Self::Expenses => AccountIdx::from(5),
        }
    }

    pub const fn first_control_account(&self) -> ControlAccountPath {
        ControlAccountPath {
            category: *self,
            index: AccountIdx::FIRST,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlAccountPath {
    pub category: ChartCategory,
    pub index: AccountIdx,
}

impl std::fmt::Display for ControlAccountPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:0<ENCODED_PATH_WIDTH$}",
            format!("{:01}{:02}", self.category.index(), self.index)
        )
    }
}

impl ControlAccountPath {
    pub fn next(&self) -> Result<Self, ChartPathError> {
        let next_index = self.index.next();
        if next_index > AccountIdx::MAX_TWO_DIGIT {
            Err(ChartPathError::ControlIndexOverflowForCategory(
                self.category,
            ))
        } else {
            Ok(Self {
                category: self.category,
                index: next_index,
            })
        }
    }

    pub fn path_encode(&self, chart_id: ChartId) -> String {
        format!("{}::{}", chart_id, self)
    }

    pub const fn first_control_sub_account(&self) -> ControlSubAccountPath {
        ControlSubAccountPath {
            category: self.category,
            control_index: self.index,
            index: AccountIdx::FIRST,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlSubAccountPath {
    pub category: ChartCategory,
    pub control_index: AccountIdx,
    pub index: AccountIdx,
}

impl std::fmt::Display for ControlSubAccountPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:0<ENCODED_PATH_WIDTH$}",
            format!(
                "{:01}{:02}{:02}",
                self.category.index(),
                self.control_index,
                self.index
            )
        )
    }
}

impl ControlSubAccountPath {
    pub fn next(&self) -> Result<Self, ChartPathError> {
        let next_index = self.index.next();
        if next_index > AccountIdx::MAX_TWO_DIGIT {
            Err(ChartPathError::ControlSubIndexOverflowForControlAccount(
                self.category,
                self.control_index,
            ))
        } else {
            Ok(Self {
                category: self.category,
                control_index: self.control_index,
                index: next_index,
            })
        }
    }

    pub fn path_encode(&self, chart_id: ChartId) -> String {
        format!("{}::{}", chart_id, self)
    }

    pub fn normal_balance_type(&self) -> DebitOrCredit {
        match self.category {
            ChartCategory::Assets | ChartCategory::Expenses => DebitOrCredit::Debit,
            _ => DebitOrCredit::Credit,
        }
    }

    pub fn control_account(&self) -> ControlAccountPath {
        ControlAccountPath {
            category: self.category,
            index: self.control_index,
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
            let path = ChartCategory::Assets;
            assert_eq!(path.to_string(), "10000");
        }

        #[test]
        fn test_control_account_formatting() {
            let path = ControlAccountPath {
                category: ChartCategory::Liabilities,
                index: 1.into(),
            };
            assert_eq!(path.to_string(), "20100");
        }

        #[test]
        fn test_control_sub_account_formatting() {
            let path = ControlSubAccountPath {
                category: ChartCategory::Equity,
                control_index: 1.into(),
                index: 2.into(),
            };
            assert_eq!(path.to_string(), "30102");
        }
    }

    mod control_account_extraction_tests {
        use super::*;

        const CATEGORY: ChartCategory = ChartCategory::Assets;
        const CONTROL_INDEX: AccountIdx = AccountIdx::FIRST;
        const EXPECTED: ControlAccountPath = ControlAccountPath {
            category: CATEGORY,
            index: CONTROL_INDEX,
        };

        #[test]
        fn test_control_account_from_control_sub_account() {
            let sub_account = ControlSubAccountPath {
                category: CATEGORY,
                control_index: CONTROL_INDEX,
                index: 2.into(),
            };

            assert_eq!(sub_account.control_account(), EXPECTED);
        }
    }

    mod first_account_create {
        use super::*;

        #[test]
        fn test_first_control_account_creation() {
            let category = ChartCategory::Assets;
            let control = category.first_control_account();

            assert_eq!(
                control,
                ControlAccountPath {
                    category: ChartCategory::Assets,
                    index: AccountIdx::FIRST,
                }
            );
        }

        #[test]
        fn test_first_control_sub_account_creation() {
            let control_account = ControlAccountPath {
                category: ChartCategory::Assets,
                index: AccountIdx::FIRST,
            };

            let sub = control_account.first_control_sub_account();
            assert_eq!(
                sub,
                ControlSubAccountPath {
                    category: ChartCategory::Assets,
                    control_index: AccountIdx::FIRST,
                    index: AccountIdx::FIRST,
                }
            );
        }
    }

    mod next_account_create {
        use super::*;

        #[test]
        fn test_next_control_account_success() {
            let control_account = ControlAccountPath {
                category: ChartCategory::Assets,
                index: 1.into(),
            };

            let next_control = control_account.next().unwrap();
            assert_eq!(
                next_control,
                ControlAccountPath {
                    category: ChartCategory::Assets,
                    index: 2.into(),
                }
            );
        }

        #[test]
        fn test_next_control_account_overflow() {
            let max_control = ControlAccountPath {
                category: ChartCategory::Assets,
                index: AccountIdx::MAX_TWO_DIGIT,
            };
            assert!(max_control.next().is_err());
        }

        #[test]
        fn test_next_control_sub_account_success() {
            let sub = ControlSubAccountPath {
                category: ChartCategory::Assets,
                control_index: 1.into(),
                index: 1.into(),
            };

            let next_sub = sub.next().unwrap();
            assert_eq!(
                next_sub,
                ControlSubAccountPath {
                    category: ChartCategory::Assets,
                    control_index: 1.into(),
                    index: 2.into(),
                }
            );
        }

        #[test]
        fn test_next_control_sub_account_overflow() {
            let max_sub = ControlSubAccountPath {
                category: ChartCategory::Assets,
                control_index: 1.into(),
                index: AccountIdx::MAX_TWO_DIGIT,
            };
            assert!(max_sub.next().is_err());
        }
    }
}
