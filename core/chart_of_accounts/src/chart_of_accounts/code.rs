use serde::{Deserialize, Serialize};

use crate::primitives::{AccountIdx, ChartOfAccountCategoryCode};

use super::error::*;

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
    type Err = ChartOfAccountError;

    fn from_str(s: &str) -> Result<Self, ChartOfAccountError> {
        if s.len() != 8 {
            return Err(ChartOfAccountError::InvalidCodeLength(s.to_string()));
        }

        fn parse_segment(s: &str) -> Result<u32, ChartOfAccountError> {
            Ok(s.parse::<u32>()?)
        }

        let category_segment = parse_segment(&s[0..1])?;
        let category = Self::category_from_number(category_segment)
            .ok_or(ChartOfAccountError::InvalidCategoryNumber(category_segment))?;

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
            _ => Err(ChartOfAccountError::InvalidCodeString(s.to_string())),
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
    ) -> Result<Self, ChartOfAccountError> {
        match category {
            Self::Category(category) => Ok(Self::ControlAccount {
                category,
                index: AccountIdx::FIRST,
            }),
            _ => Err(ChartOfAccountError::InvalidCategoryCodeForNewControlAccount),
        }
    }

    pub fn first_control_sub_account(control_account: &Self) -> Result<Self, ChartOfAccountError> {
        match control_account {
            Self::ControlAccount { category, index } => Ok(Self::ControlSubAccount {
                category: *category,
                control_index: *index,
                index: AccountIdx::FIRST,
            }),
            _ => Err(ChartOfAccountError::InvalidControlAccountCodeForNewControlSubAccount),
        }
    }

    pub fn first_transaction_account(
        control_sub_account: &Self,
    ) -> Result<Self, ChartOfAccountError> {
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
            _ => Err(ChartOfAccountError::InvalidSubControlAccountCodeForNewTransactionAccount),
        }
    }

    pub fn next(&self) -> Result<Self, ChartOfAccountError> {
        match *self {
            Self::Category(_) => Ok(*self), // Categories don't have next
            Self::ControlAccount { category, index } => {
                let next_index = index.next();
                if next_index > AccountIdx::MAX_TWO_DIGIT {
                    Err(ChartOfAccountError::ControlIndexOverflowForCategory(
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
                        ChartOfAccountError::ControlSubIndexOverflowForControlAccount(
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
                        ChartOfAccountError::TransactionIndexOverflowForControlSubAccount(
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
