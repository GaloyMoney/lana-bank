use chrono::{DateTime, Datelike, Utc};
use derive_builder::Builder;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    entity::*,
    ledger::{loan::LoanAccountIds, user::UserLedgerAccountIds},
    primitives::*,
};

use super::error::LoanError;
use super::terms::TermValues;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LoanEvent {
    Initialized {
        id: LoanId,
        user_id: UserId,
        user_account_ids: UserLedgerAccountIds,
        principal: UsdCents,
        initial_collateral: Satoshis,
        terms: TermValues,
        account_ids: LoanAccountIds,
        start_date: DateTime<Utc>,
    },
    Collateralized {
        tx_id: LedgerTxId,
    },
    InterestRecorded {
        tx_id: LedgerTxId,
        tx_ref: String,
    },
    Completed {
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
    },
}

impl EntityEvent for LoanEvent {
    type EntityId = LoanId;
    fn event_table_name() -> &'static str {
        "loan_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Loan {
    pub id: LoanId,
    pub user_id: UserId,
    pub terms: TermValues,
    pub account_ids: LoanAccountIds,
    pub user_account_ids: UserLedgerAccountIds,
    pub start_date: DateTime<Utc>,
    pub(super) events: EntityEvents<LoanEvent>,
}

impl Loan {
    pub fn initial_collateral(&self) -> Satoshis {
        if let Some(LoanEvent::Initialized {
            initial_collateral, ..
        }) = self.events.iter().next()
        {
            *initial_collateral
        } else {
            unreachable!("Initialized event not found")
        }
    }

    pub fn initial_principal(&self) -> UsdCents {
        if let Some(LoanEvent::Initialized { principal, .. }) = self.events.iter().next() {
            *principal
        } else {
            unreachable!("Initialized event not found")
        }
    }

    pub fn is_collateralized(&self) -> bool {
        for event in self.events.iter() {
            match event {
                LoanEvent::Collateralized { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    pub(super) fn collateralize(&mut self, tx_id: LedgerTxId) {
        self.events.push(LoanEvent::Collateralized { tx_id });
    }

    pub fn next_interest_at(&self) -> Option<DateTime<Utc>> {
        if !self.is_completed() && !self.is_expired() {
            self.terms
                .interval
                .next_interest_payment(chrono::Utc::now())
        } else {
            None
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now() > self.terms.duration.expiration_date(self.start_date)
    }

    fn count_interest_incurred(&self) -> usize {
        self.events
            .iter()
            .filter(|event| matches!(event, LoanEvent::InterestRecorded { .. }))
            .count()
    }

    pub fn is_completed(&self) -> bool {
        self.events
            .iter()
            .any(|event| matches!(event, LoanEvent::Completed { .. }))
    }

    pub fn calculate_interest(&self) -> UsdCents {
        let principal = Decimal::from(self.initial_principal().into_inner());
        let daily_rate = self.terms.daily_rate();
        let days = if self.count_interest_incurred() == 0 {
            let next_payment = self
                .terms
                .interval
                .next_interest_payment(self.start_date)
                .expect("should return an interest payment date");
            next_payment.day() - self.start_date.day()
        } else {
            self.terms
                .interval
                .next_interest_payment(Utc::now())
                .expect("should return an interest payment date")
                .day()
        };

        let interest = (daily_rate * principal * Decimal::from(days)).ceil();

        UsdCents::from(
            interest
                .to_u64()
                .expect("interest amount should be a positive integer"),
        )
    }

    pub fn record_incur_interest_transaction(
        &mut self,
        tx_id: LedgerTxId,
    ) -> Result<String, LoanError> {
        if self.is_completed() {
            return Err(LoanError::AlreadyCompleted);
        }

        let tx_ref = format!(
            "{}-interest-{}",
            self.id,
            self.count_interest_incurred() + 1
        );
        self.events.push(LoanEvent::InterestRecorded {
            tx_id,
            tx_ref: tx_ref.clone(),
        });
        Ok(tx_ref)
    }
}

impl Entity for Loan {
    type Event = LoanEvent;
}

impl TryFrom<EntityEvents<LoanEvent>> for Loan {
    type Error = EntityError;

    fn try_from(events: EntityEvents<LoanEvent>) -> Result<Self, Self::Error> {
        let mut builder = LoanBuilder::default();
        for event in events.iter() {
            match event {
                LoanEvent::Initialized {
                    id,
                    user_id,
                    terms,
                    account_ids,
                    user_account_ids,
                    start_date,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .user_id(*user_id)
                        .terms(terms.clone())
                        .account_ids(*account_ids)
                        .user_account_ids(*user_account_ids)
                        .start_date(*start_date);
                }
                LoanEvent::Collateralized { .. } => (),
                LoanEvent::InterestRecorded { .. } => (),
                LoanEvent::Completed { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewLoan {
    #[builder(setter(into))]
    pub(super) id: LoanId,
    #[builder(setter(into))]
    pub(super) user_id: UserId,
    terms: TermValues,
    principal: UsdCents,
    initial_collateral: Satoshis,
    account_ids: LoanAccountIds,
    user_account_ids: UserLedgerAccountIds,
    #[builder(default = "Utc::now()")]
    start_date: DateTime<Utc>,
}

impl NewLoan {
    pub fn builder() -> NewLoanBuilder {
        NewLoanBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<LoanEvent> {
        EntityEvents::init(
            self.id,
            [LoanEvent::Initialized {
                id: self.id,
                user_id: self.user_id,
                terms: self.terms,
                principal: self.principal,
                initial_collateral: self.initial_collateral,
                account_ids: self.account_ids,
                user_account_ids: self.user_account_ids,
                start_date: self.start_date,
            }],
        )
    }
}
