mod entry;
pub mod error;
mod jobs;
mod repo;

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use obix::EventSequence;
use obix::out::{Outbox, OutboxEventJobConfig, OutboxEventMarker};

use crate::{CoreCreditCollectionEvent, CoreCreditEvent, primitives::*};
use audit::AuditSvc;
use authz::PermissionCheck;
use error::CreditFacilityRepaymentPlanError;
use jobs::credit_facility_repayment_plan;
use tracing::instrument;
use tracing_macros::record_error_severity;

pub use entry::*;
pub use repo::RepaymentPlanRepo;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreditFacilityRepaymentPlan {
    facility_amount: UsdCents,
    terms: Option<TermValues>,
    activated_at: Option<DateTime<Utc>>,
    last_interest_accrual_at: Option<DateTime<Utc>>,
    last_updated_on_sequence: EventSequence,

    pub entries: Vec<CreditFacilityRepaymentPlanEntry>,

    #[serde(default)]
    applied_allocations: HashSet<PaymentAllocationId>,
    #[serde(default)]
    applied_accruals: HashSet<LedgerTxId>,
}

impl CreditFacilityRepaymentPlan {
    fn existing_obligations(&self) -> Vec<CreditFacilityRepaymentPlanEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.is_not_upcoming())
            .cloned()
            .collect()
    }

    fn planned_disbursals(&self, now: DateTime<Utc>) -> Vec<CreditFacilityRepaymentPlanEntry> {
        let terms = self.terms.expect("Missing FacilityCreated event");
        let facility_amount = self.facility_amount;
        let structuring_fee = terms.one_time_fee_rate.apply(facility_amount);

        let activated_at = self.activated_at.unwrap_or(now);
        let maturity_date = terms.maturity_date(activated_at);

        let mut disbursals = vec![];
        if !structuring_fee.is_zero() {
            disbursals.push(CreditFacilityRepaymentPlanEntry {
                repayment_type: RepaymentType::Disbursal,
                obligation_id: None,
                status: RepaymentStatus::Upcoming,

                initial: structuring_fee,
                outstanding: structuring_fee,

                due_at: maturity_date,
                overdue_at: None,
                defaulted_at: None,
                recorded_at: activated_at,
                effective: activated_at.date_naive(),
            })
        }
        disbursals.push(CreditFacilityRepaymentPlanEntry {
            repayment_type: RepaymentType::Disbursal,
            obligation_id: None,
            status: RepaymentStatus::Upcoming,

            initial: facility_amount,
            outstanding: facility_amount,

            due_at: maturity_date,
            overdue_at: None,
            defaulted_at: None,
            recorded_at: activated_at,
            effective: activated_at.date_naive(),
        });

        disbursals
    }

    fn planned_interest_accruals(
        &self,
        updated_entries: &[CreditFacilityRepaymentPlanEntry],
        now: DateTime<Utc>,
    ) -> Vec<CreditFacilityRepaymentPlanEntry> {
        let terms = self.terms.expect("Missing FacilityCreated event");
        let activated_at = self.activated_at.unwrap_or(now);

        let maturity_date = terms.maturity_date(activated_at);
        let mut next_interest_period =
            if let Some(last_interest_payment) = self.last_interest_accrual_at {
                terms
                    .accrual_cycle_interval
                    .period_from(last_interest_payment)
                    .next()
                    .truncate(maturity_date.start_of_day())
            } else {
                terms
                    .accrual_cycle_interval
                    .period_from(activated_at)
                    .truncate(maturity_date.start_of_day())
            };

        let disbursed_outstanding = updated_entries
            .iter()
            .filter_map(|entry| match entry {
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Disbursal,
                    outstanding,
                    ..
                } => Some(*outstanding),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, outstanding| acc + outstanding);

        let mut planned_interest_entries = vec![];
        while let Some(period) = next_interest_period {
            let interest = terms
                .annual_rate
                .interest_for_time_period(disbursed_outstanding, period.days());

            planned_interest_entries.push(CreditFacilityRepaymentPlanEntry {
                repayment_type: RepaymentType::Interest,
                obligation_id: None,
                status: RepaymentStatus::Upcoming,
                initial: interest,
                outstanding: interest,

                due_at: EffectiveDate::from(period.end),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: period.end,
                effective: period.end.date_naive(),
            });

            next_interest_period = period.next().truncate(maturity_date.start_of_day());
        }

        planned_interest_entries
    }

    pub(crate) fn process_credit_event(
        &mut self,
        sequence: EventSequence,
        event: &CoreCreditEvent,
        now: DateTime<Utc>,
        recorded_at: DateTime<Utc>,
    ) -> bool {
        self.last_updated_on_sequence = sequence;

        let mut existing_obligations = self.existing_obligations();

        match event {
            CoreCreditEvent::FacilityProposalCreated { entity } => {
                self.terms = Some(entity.terms);
                self.facility_amount = entity.amount;
            }
            CoreCreditEvent::FacilityActivated { entity } => {
                self.activated_at = Some(entity.activated_at);
            }
            CoreCreditEvent::AccrualPosted { entity } => {
                let posting = entity
                    .posting
                    .as_ref()
                    .expect("posting must be set for AccrualPosted");
                if !posting.amount.is_zero() {
                    return false;
                }
                // Skip if already processed (idempotent for replay)
                if !self.applied_accruals.insert(posting.tx_id) {
                    return false;
                }

                let entry = CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    obligation_id: None,
                    status: RepaymentStatus::Paid,

                    initial: UsdCents::ZERO,
                    outstanding: UsdCents::ZERO,

                    due_at: entity.due_at,
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at,
                    effective: posting.effective,
                };

                let effective = EffectiveDate::from(posting.effective);
                self.last_interest_accrual_at = Some(effective.end_of_day());

                existing_obligations.push(entry);
            }
            _ => return false,
        };

        self.rebuild_entries(existing_obligations, now);
        true
    }

    pub(crate) fn process_collection_event(
        &mut self,
        sequence: EventSequence,
        event: &CoreCreditCollectionEvent,
        now: DateTime<Utc>,
    ) -> bool {
        self.last_updated_on_sequence = sequence;

        let mut existing_obligations = self.existing_obligations();

        match event {
            CoreCreditCollectionEvent::ObligationCreated { entity } => {
                if existing_obligations
                    .iter()
                    .any(|e| e.obligation_id == Some(entity.id))
                {
                    return false;
                }

                let entry = CreditFacilityRepaymentPlanEntry {
                    repayment_type: (&entity.obligation_type).into(),
                    obligation_id: Some(entity.id),
                    status: RepaymentStatus::NotYetDue,

                    initial: entity.initial_amount,
                    outstanding: entity.outstanding_amount,

                    due_at: entity.due_at,
                    overdue_at: entity.overdue_at,
                    defaulted_at: entity.defaulted_at,
                    recorded_at: entity.recorded_at,
                    effective: entity.effective,
                };
                if entity.obligation_type == ObligationType::Interest {
                    let effective = EffectiveDate::from(entity.effective);
                    self.last_interest_accrual_at = Some(effective.end_of_day());
                }

                existing_obligations.push(entry);
            }
            CoreCreditCollectionEvent::PaymentAllocationCreated { entity } => {
                if !self.applied_allocations.insert(entity.id) {
                    return false;
                }

                if let Some(entry) = existing_obligations.iter_mut().find_map(|entry| {
                    (entry.obligation_id == Some(entity.obligation_id)).then_some(entry)
                }) {
                    entry.outstanding -= entity.amount;
                } else {
                    return false;
                }
            }
            CoreCreditCollectionEvent::ObligationDue { entity }
            | CoreCreditCollectionEvent::ObligationOverdue { entity }
            | CoreCreditCollectionEvent::ObligationDefaulted { entity }
            | CoreCreditCollectionEvent::ObligationCompleted { entity } => {
                if let Some(entry) = existing_obligations
                    .iter_mut()
                    .find_map(|entry| (entry.obligation_id == Some(entity.id)).then_some(entry))
                {
                    entry.status = match event {
                        CoreCreditCollectionEvent::ObligationDue { .. } => RepaymentStatus::Due,
                        CoreCreditCollectionEvent::ObligationOverdue { .. } => {
                            RepaymentStatus::Overdue
                        }
                        CoreCreditCollectionEvent::ObligationDefaulted { .. } => {
                            RepaymentStatus::Defaulted
                        }
                        CoreCreditCollectionEvent::ObligationCompleted { .. } => {
                            RepaymentStatus::Paid
                        }
                        _ => unreachable!(),
                    };
                } else {
                    return false;
                }
            }
            _ => return false,
        };

        self.rebuild_entries(existing_obligations, now);
        true
    }

    fn rebuild_entries(
        &mut self,
        existing_obligations: Vec<CreditFacilityRepaymentPlanEntry>,
        now: DateTime<Utc>,
    ) {
        let updated_entries = if !existing_obligations.is_empty() {
            existing_obligations
        } else {
            self.planned_disbursals(now)
        };

        let planned_interest_entries = self.planned_interest_accruals(&updated_entries, now);

        self.entries = updated_entries
            .into_iter()
            .chain(planned_interest_entries)
            .collect();
        self.entries.sort();
    }
}

pub struct RepaymentPlans<Perms> {
    repo: Arc<RepaymentPlanRepo>,
    authz: Arc<Perms>,
}

impl<Perms> Clone for RepaymentPlans<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms> RepaymentPlans<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    pub async fn init<E>(
        pool: &sqlx::PgPool,
        outbox: &Outbox<E>,
        jobs: &mut job::Jobs,
        authz: Arc<Perms>,
    ) -> Result<Self, CreditFacilityRepaymentPlanError>
    where
        E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<crate::CoreCreditCollectionEvent>,
    {
        let repo = Arc::new(RepaymentPlanRepo::new(pool));

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(
                    credit_facility_repayment_plan::REPAYMENT_PLAN_PROJECTION,
                ),
                credit_facility_repayment_plan::RepaymentPlanProjectionHandler::new(repo.clone()),
            )
            .await?;

        Ok(Self { repo, authz })
    }

    #[record_error_severity]
    #[instrument(name = "credit.repayment_plan", skip(self, credit_facility_id), fields(credit_facility_id = tracing::field::Empty))]
    pub async fn find_for_credit_facility_id<T: From<CreditFacilityRepaymentPlanEntry>>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Vec<T>, CreditFacilityRepaymentPlanError> {
        let id = credit_facility_id.into();
        tracing::Span::current().record("credit_facility_id", tracing::field::display(id));

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;
        let repayment_plan = self.repo.load(id).await?;
        Ok(repayment_plan.entries.into_iter().map(T::from).collect())
    }

    pub(crate) async fn find_for_credit_facility_id_without_audit(
        &self,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Vec<CreditFacilityRepaymentPlanEntry>, CreditFacilityRepaymentPlanError> {
        let repayment_plan = self.repo.load(credit_facility_id.into()).await?;
        Ok(repayment_plan.entries.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::inconsistent_digit_grouping)]

    use rust_decimal_macros::dec;

    use crate::{
        CreditFacilityReceivable, DisbursalPolicy, FacilityCollateralization, FacilityDuration,
        InterestInterval, ObligationDuration, OneTimeFeeRatePct,
    };
    use core_credit_collection::{PublicObligation, PublicPaymentAllocation};

    use super::*;

    #[derive(Debug, Default, PartialEq, Eq)]
    struct EntriesCount {
        interest_unpaid: usize,
        interest_paid: usize,
        interest_upcoming: usize,
        disbursals_unpaid: usize,
        disbursals_paid: usize,
        disbursals_upcoming: usize,
    }

    fn terms(one_time_fee_rate: u64) -> TermValues {
        let one_time_fee_rate = OneTimeFeeRatePct::new(one_time_fee_rate);
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(FacilityDuration::Months(3))
            .interest_due_duration_from_accrual(ObligationDuration::Days(0))
            .obligation_overdue_duration_from_due(None)
            .obligation_liquidation_duration_from_due(None)
            .accrual_cycle_interval(InterestInterval::EndOfMonth)
            .accrual_interval(InterestInterval::EndOfDay)
            .one_time_fee_rate(one_time_fee_rate)
            .disbursal_policy(DisbursalPolicy::SingleDisbursal)
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    fn default_start_date() -> DateTime<Utc> {
        "2021-01-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap()
    }

    fn default_start_date_with_days(days: i64) -> DateTime<Utc> {
        "2021-01-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap() + chrono::Duration::days(days)
    }

    fn default_facility_amount() -> UsdCents {
        UsdCents::from(1_000_000_00)
    }

    fn default_collateralization() -> FacilityCollateralization {
        FacilityCollateralization {
            state: CollateralizationState::FullyCollateralized,
            collateral: Satoshis::from(1_000_000),
            outstanding: CreditFacilityReceivable {
                disbursed: default_facility_amount(),
                interest: UsdCents::ZERO,
            },
            price_at_state_change: PriceOfOneBTC::new(UsdCents::from(5_000_000)),
        }
    }

    fn plan(terms: TermValues) -> CreditFacilityRepaymentPlan {
        let mut plan = CreditFacilityRepaymentPlan::default();
        plan.process_credit_event(
            Default::default(),
            &CoreCreditEvent::FacilityProposalCreated {
                entity: crate::PublicCreditFacilityProposal {
                    id: CreditFacilityProposalId::new(),
                    status: CreditFacilityProposalStatus::PendingCustomerApproval,
                    terms,
                    amount: default_facility_amount(),
                    customer_id: CustomerId::new(),
                    created_at: default_start_date(),
                },
            },
            default_start_date(),
            default_start_date(),
        );

        plan
    }

    fn initial_plan() -> CreditFacilityRepaymentPlan {
        plan(terms(5))
    }

    fn initial_plan_no_structuring_fee() -> CreditFacilityRepaymentPlan {
        plan(terms(0))
    }

    fn accrual_posted_event(period: InterestPeriod) -> CoreCreditEvent {
        CoreCreditEvent::AccrualPosted {
            entity: crate::PublicInterestAccrualCycle {
                id: InterestAccrualCycleId::new(),
                credit_facility_id: CreditFacilityId::new(),
                period,
                due_at: EffectiveDate::from(period.end),
                posting: Some(
                    crate::credit_facility::interest_accrual_cycle::AccrualPosting {
                        tx_id: LedgerTxId::new(),
                        amount: UsdCents::ZERO,
                        effective: period.end.date_naive(),
                    },
                ),
            },
        }
    }

    fn process_credit_events(plan: &mut CreditFacilityRepaymentPlan, events: Vec<CoreCreditEvent>) {
        for event in events {
            plan.process_credit_event(
                Default::default(),
                &event,
                default_start_date(),
                default_start_date(),
            );
        }
    }

    enum TestEvent {
        Credit(CoreCreditEvent),
        Collection(CoreCreditCollectionEvent),
    }

    fn process_test_events(plan: &mut CreditFacilityRepaymentPlan, events: Vec<TestEvent>) {
        for event in events {
            match &event {
                TestEvent::Credit(e) => {
                    plan.process_credit_event(
                        Default::default(),
                        e,
                        default_start_date(),
                        default_start_date(),
                    );
                }
                TestEvent::Collection(e) => {
                    plan.process_collection_event(Default::default(), e, default_start_date());
                }
            }
        }
    }

    fn count_entries(plan: &CreditFacilityRepaymentPlan) -> EntriesCount {
        let mut res = EntriesCount::default();

        for entry in plan.entries.iter() {
            match entry {
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Disbursal,
                    status: RepaymentStatus::Upcoming,
                    ..
                } => res.disbursals_upcoming += 1,
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Disbursal,
                    status: RepaymentStatus::Paid,
                    ..
                } => res.disbursals_paid += 1,
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Disbursal,
                    ..
                } => res.disbursals_unpaid += 1,
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    status: RepaymentStatus::Upcoming,
                    ..
                } => res.interest_upcoming += 1,
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    status: RepaymentStatus::Paid,
                    ..
                } => res.interest_paid += 1,
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    ..
                } => res.interest_unpaid += 1,
            }
        }

        res
    }

    #[test]
    fn facility_created() {
        let plan = initial_plan();
        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 0,
                interest_paid: 0,
                interest_upcoming: 4,
                disbursals_unpaid: 0,
                disbursals_paid: 0,
                disbursals_upcoming: 2,
            }
        );
    }

    #[test]
    fn with_zero_structuring_fee() {
        let mut plan = initial_plan_no_structuring_fee();

        let events = vec![CoreCreditEvent::FacilityActivated {
            entity: crate::PublicCreditFacility {
                id: CreditFacilityId::new(),
                customer_id: CustomerId::new(),
                collateral_id: CollateralId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
                completed_at: None,
                liquidation_trigger: None,
                collateralization: default_collateralization(),
            },
        }];
        process_credit_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 0,
                interest_paid: 0,
                interest_upcoming: 4,
                disbursals_unpaid: 0,
                disbursals_paid: 0,
                disbursals_upcoming: 1,
            }
        );
    }

    #[test]
    fn with_zero_structuring_fee_and_first_accrual() {
        let mut plan = initial_plan_no_structuring_fee();

        let period = InterestInterval::EndOfMonth.period_from(default_start_date());
        let events = vec![
            CoreCreditEvent::FacilityActivated {
                entity: crate::PublicCreditFacility {
                    id: CreditFacilityId::new(),
                    customer_id: CustomerId::new(),
                    collateral_id: CollateralId::new(),
                    activation_tx_id: LedgerTxId::new(),
                    activated_at: default_start_date(),
                    amount: default_facility_amount(),
                    completed_at: None,
                    liquidation_trigger: None,
                    collateralization: default_collateralization(),
                },
            },
            accrual_posted_event(period),
        ];
        process_credit_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 0,
                interest_paid: 1,
                interest_upcoming: 3,
                disbursals_unpaid: 0,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );
    }

    #[test]
    fn with_zero_structuring_fee_and_second_accrual() {
        let mut plan = initial_plan_no_structuring_fee();

        let period_1 = InterestInterval::EndOfMonth.period_from(default_start_date());
        let period_2 = period_1.next();
        let events = vec![
            CoreCreditEvent::FacilityActivated {
                entity: crate::PublicCreditFacility {
                    id: CreditFacilityId::new(),
                    customer_id: CustomerId::new(),
                    collateral_id: CollateralId::new(),
                    activation_tx_id: LedgerTxId::new(),
                    activated_at: default_start_date(),
                    amount: default_facility_amount(),
                    completed_at: None,
                    liquidation_trigger: None,
                    collateralization: default_collateralization(),
                },
            },
            accrual_posted_event(period_1),
            accrual_posted_event(period_2),
        ];
        process_credit_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 0,
                interest_paid: 2,
                interest_upcoming: 2,
                disbursals_unpaid: 0,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );
    }

    #[test]
    fn with_first_disbursal_obligation_created() {
        let mut plan = initial_plan();

        let recorded_at = default_start_date();
        let events = vec![
            TestEvent::Credit(CoreCreditEvent::FacilityActivated {
                entity: crate::PublicCreditFacility {
                    id: CreditFacilityId::new(),
                    customer_id: CustomerId::new(),
                    collateral_id: CollateralId::new(),
                    activation_tx_id: LedgerTxId::new(),
                    activated_at: default_start_date(),
                    amount: default_facility_amount(),
                    completed_at: None,
                    liquidation_trigger: None,
                    collateralization: default_collateralization(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Disbursal,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(100_000_00),
                    outstanding_amount: UsdCents::from(100_000_00),
                    due_at: EffectiveDate::from(default_start_date()),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at,
                    effective: recorded_at.date_naive(),
                },
            }),
        ];
        process_test_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 0,
                interest_paid: 0,
                interest_upcoming: 4,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );
    }

    #[test]
    fn with_first_interest_obligation_created() {
        let mut plan = initial_plan();

        let disbursal_recorded_at = default_start_date();
        let interest_recorded_at = default_start_date_with_days(30);
        let events = vec![
            TestEvent::Credit(CoreCreditEvent::FacilityActivated {
                entity: crate::PublicCreditFacility {
                    id: CreditFacilityId::new(),
                    customer_id: CustomerId::new(),
                    collateral_id: CollateralId::new(),
                    activation_tx_id: LedgerTxId::new(),
                    activated_at: default_start_date(),
                    amount: default_facility_amount(),
                    completed_at: None,
                    liquidation_trigger: None,
                    collateralization: default_collateralization(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Disbursal,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(100_000_00),
                    outstanding_amount: UsdCents::from(100_000_00),
                    due_at: EffectiveDate::from(disbursal_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: disbursal_recorded_at,
                    effective: disbursal_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(1_000_00),
                    outstanding_amount: UsdCents::from(1_000_00),
                    due_at: EffectiveDate::from(interest_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: interest_recorded_at,
                    effective: interest_recorded_at.date_naive(),
                },
            }),
        ];
        process_test_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 1,
                interest_paid: 0,
                interest_upcoming: 3,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );
    }

    #[test]
    fn with_first_interest_partial_payment() {
        let interest_obligation_id = ObligationId::new();

        let mut plan = initial_plan();

        let disbursal_recorded_at = default_start_date();
        let interest_recorded_at = default_start_date_with_days(30);
        let events = vec![
            TestEvent::Credit(CoreCreditEvent::FacilityActivated {
                entity: crate::PublicCreditFacility {
                    id: CreditFacilityId::new(),
                    customer_id: CustomerId::new(),
                    collateral_id: CollateralId::new(),
                    activation_tx_id: LedgerTxId::new(),
                    activated_at: default_start_date(),
                    amount: default_facility_amount(),
                    completed_at: None,
                    liquidation_trigger: None,
                    collateralization: default_collateralization(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Disbursal,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(100_000_00),
                    outstanding_amount: UsdCents::from(100_000_00),
                    due_at: EffectiveDate::from(disbursal_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: disbursal_recorded_at,
                    effective: disbursal_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: interest_obligation_id,
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(1_000_00),
                    outstanding_amount: UsdCents::from(1_000_00),
                    due_at: EffectiveDate::from(interest_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: interest_recorded_at,
                    effective: interest_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::PaymentAllocationCreated {
                entity: PublicPaymentAllocation {
                    id: PaymentAllocationId::new(),
                    obligation_id: interest_obligation_id,
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    amount: UsdCents::from(400_00),
                    recorded_at: interest_recorded_at,
                    effective: interest_recorded_at.date_naive(),
                },
            }),
        ];
        process_test_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 1,
                interest_paid: 0,
                interest_upcoming: 3,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );

        let interest_entry_outstanding = plan
            .entries
            .iter()
            .find_map(|e| match e {
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    obligation_id: Some(_),
                    outstanding,
                    ..
                } => Some(outstanding),
                _ => None,
            })
            .unwrap();
        assert_eq!(*interest_entry_outstanding, UsdCents::from(600_00));
    }

    #[test]
    fn with_first_interest_paid() {
        let interest_obligation_id = ObligationId::new();

        let mut plan = initial_plan();

        let disbursal_recorded_at = default_start_date();
        let interest_recorded_at = default_start_date_with_days(30);
        let events = vec![
            TestEvent::Credit(CoreCreditEvent::FacilityActivated {
                entity: crate::PublicCreditFacility {
                    id: CreditFacilityId::new(),
                    customer_id: CustomerId::new(),
                    collateral_id: CollateralId::new(),
                    activation_tx_id: LedgerTxId::new(),
                    activated_at: default_start_date(),
                    amount: default_facility_amount(),
                    completed_at: None,
                    liquidation_trigger: None,
                    collateralization: default_collateralization(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Disbursal,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(100_000_00),
                    outstanding_amount: UsdCents::from(100_000_00),
                    due_at: EffectiveDate::from(disbursal_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: disbursal_recorded_at,
                    effective: disbursal_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: interest_obligation_id,
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(1_000_00),
                    outstanding_amount: UsdCents::from(1_000_00),
                    due_at: EffectiveDate::from(interest_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: interest_recorded_at,
                    effective: interest_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::PaymentAllocationCreated {
                entity: PublicPaymentAllocation {
                    id: PaymentAllocationId::new(),
                    obligation_id: interest_obligation_id,
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    amount: UsdCents::from(1_000_00),
                    recorded_at: interest_recorded_at,
                    effective: interest_recorded_at.date_naive(),
                },
            }),
        ];
        process_test_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 1,
                interest_paid: 0,
                interest_upcoming: 3,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );

        let (outstanding, status) = plan
            .entries
            .iter()
            .find_map(|e| match e {
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    obligation_id: Some(_),
                    outstanding,
                    status,
                    ..
                } => Some((outstanding, status)),
                _ => None,
            })
            .unwrap();
        assert_eq!(*outstanding, UsdCents::ZERO);
        assert_ne!(*status, RepaymentStatus::Paid);

        plan.process_collection_event(
            Default::default(),
            &CoreCreditCollectionEvent::ObligationCompleted {
                entity: PublicObligation {
                    id: interest_obligation_id,
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(1_000_00),
                    outstanding_amount: UsdCents::ZERO,
                    due_at: EffectiveDate::from(interest_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: interest_recorded_at,
                    effective: interest_recorded_at.date_naive(),
                },
            },
            default_start_date(),
        );
        let interest_entry_status = plan
            .entries
            .iter()
            .find_map(|e| match e {
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    obligation_id: Some(_),
                    status,
                    ..
                } => Some(status),
                _ => None,
            })
            .unwrap();
        assert_eq!(*interest_entry_status, RepaymentStatus::Paid);
    }

    #[test]
    fn with_all_interest_obligations_created() {
        let mut plan = initial_plan();

        let disbursal_recorded_at = default_start_date();
        let interest_1_recorded_at = default_start_date_with_days(30);
        let interest_2_recorded_at = default_start_date_with_days(30 + 28);
        let interest_3_recorded_at = default_start_date_with_days(30 + 28 + 31);
        let interest_4_recorded_at = default_start_date_with_days(30 + 28 + 31 + 1);
        let events = vec![
            TestEvent::Credit(CoreCreditEvent::FacilityActivated {
                entity: crate::PublicCreditFacility {
                    id: CreditFacilityId::new(),
                    customer_id: CustomerId::new(),
                    collateral_id: CollateralId::new(),
                    activation_tx_id: LedgerTxId::new(),
                    activated_at: default_start_date(),
                    amount: default_facility_amount(),
                    completed_at: None,
                    liquidation_trigger: None,
                    collateralization: default_collateralization(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Disbursal,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(100_000_00),
                    outstanding_amount: UsdCents::from(100_000_00),
                    due_at: EffectiveDate::from(disbursal_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: disbursal_recorded_at,
                    effective: disbursal_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(1_000_00),
                    outstanding_amount: UsdCents::from(1_000_00),
                    due_at: EffectiveDate::from(interest_1_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: interest_1_recorded_at,
                    effective: interest_1_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(1_000_00),
                    outstanding_amount: UsdCents::from(1_000_00),
                    due_at: EffectiveDate::from(interest_2_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: interest_2_recorded_at,
                    effective: interest_2_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(1_000_00),
                    outstanding_amount: UsdCents::from(1_000_00),
                    due_at: EffectiveDate::from(interest_3_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: interest_3_recorded_at,
                    effective: interest_3_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(33_00),
                    outstanding_amount: UsdCents::from(33_00),
                    due_at: EffectiveDate::from(interest_4_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: interest_4_recorded_at,
                    effective: interest_4_recorded_at.date_naive(),
                },
            }),
        ];
        process_test_events(&mut plan, events);

        let counts = count_entries(&plan);
        assert_eq!(
            counts,
            EntriesCount {
                interest_unpaid: 4,
                interest_paid: 0,
                interest_upcoming: 0,
                disbursals_unpaid: 1,
                disbursals_paid: 0,
                disbursals_upcoming: 0,
            }
        );
    }

    #[test]
    fn replayed_payment_allocation_is_idempotent() {
        let interest_obligation_id = ObligationId::new();
        let allocation_id = PaymentAllocationId::new();

        let mut plan = initial_plan();

        let disbursal_recorded_at = default_start_date();
        let interest_recorded_at = default_start_date_with_days(30);

        let events = vec![
            TestEvent::Credit(CoreCreditEvent::FacilityActivated {
                entity: crate::PublicCreditFacility {
                    id: CreditFacilityId::new(),
                    customer_id: CustomerId::new(),
                    collateral_id: CollateralId::new(),
                    activation_tx_id: LedgerTxId::new(),
                    activated_at: default_start_date(),
                    amount: default_facility_amount(),
                    completed_at: None,
                    liquidation_trigger: None,
                    collateralization: default_collateralization(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: ObligationId::new(),
                    obligation_type: ObligationType::Disbursal,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(100_000_00),
                    outstanding_amount: UsdCents::from(100_000_00),
                    due_at: EffectiveDate::from(disbursal_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: disbursal_recorded_at,
                    effective: disbursal_recorded_at.date_naive(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation {
                    id: interest_obligation_id,
                    obligation_type: ObligationType::Interest,
                    beneficiary_id: CreditFacilityId::new().into(),
                    initial_amount: UsdCents::from(1_000_00),
                    outstanding_amount: UsdCents::from(1_000_00),
                    due_at: EffectiveDate::from(interest_recorded_at),
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: interest_recorded_at,
                    effective: interest_recorded_at.date_naive(),
                },
            }),
        ];
        process_test_events(&mut plan, events);

        let payment_event = CoreCreditCollectionEvent::PaymentAllocationCreated {
            entity: PublicPaymentAllocation {
                id: allocation_id,
                obligation_id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                beneficiary_id: CreditFacilityId::new().into(),
                amount: UsdCents::from(1_000_00),
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
        };

        // First processing should apply the payment
        let first_result =
            plan.process_collection_event(Default::default(), &payment_event, default_start_date());
        assert!(first_result);

        let outstanding_after_first = plan
            .entries
            .iter()
            .find_map(|e| match e {
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    obligation_id: Some(_),
                    outstanding,
                    ..
                } => Some(*outstanding),
                _ => None,
            })
            .unwrap();
        assert_eq!(outstanding_after_first, UsdCents::ZERO);

        // Second processing (replay) should be idempotent - no overflow, no change
        let second_result =
            plan.process_collection_event(Default::default(), &payment_event, default_start_date());
        assert!(!second_result);

        let outstanding_after_second = plan
            .entries
            .iter()
            .find_map(|e| match e {
                CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    obligation_id: Some(_),
                    outstanding,
                    ..
                } => Some(*outstanding),
                _ => None,
            })
            .unwrap();
        assert_eq!(outstanding_after_second, UsdCents::ZERO);
    }

    #[test]
    fn replayed_obligation_created_is_idempotent() {
        let obligation_id = ObligationId::new();

        let mut plan = initial_plan();

        let recorded_at = default_start_date();

        let activate_event = CoreCreditEvent::FacilityActivated {
            entity: crate::PublicCreditFacility {
                id: CreditFacilityId::new(),
                customer_id: CustomerId::new(),
                collateral_id: CollateralId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
                completed_at: None,
                liquidation_trigger: None,
                collateralization: default_collateralization(),
            },
        };
        plan.process_credit_event(
            Default::default(),
            &activate_event,
            default_start_date(),
            default_start_date(),
        );

        let obligation_event = CoreCreditCollectionEvent::ObligationCreated {
            entity: PublicObligation {
                id: obligation_id,
                obligation_type: ObligationType::Disbursal,
                beneficiary_id: CreditFacilityId::new().into(),
                initial_amount: UsdCents::from(100_000_00),
                outstanding_amount: UsdCents::from(100_000_00),
                due_at: EffectiveDate::from(recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at,
                effective: recorded_at.date_naive(),
            },
        };

        // First processing should create the obligation entry
        let first_result = plan.process_collection_event(
            Default::default(),
            &obligation_event,
            default_start_date(),
        );
        assert!(first_result);

        let count_after_first = plan
            .entries
            .iter()
            .filter(|e| e.obligation_id == Some(obligation_id))
            .count();
        assert_eq!(count_after_first, 1);

        // Second processing (replay) should be idempotent - no duplicate entry
        let second_result = plan.process_collection_event(
            Default::default(),
            &obligation_event,
            default_start_date(),
        );
        assert!(!second_result);

        let count_after_second = plan
            .entries
            .iter()
            .filter(|e| e.obligation_id == Some(obligation_id))
            .count();
        assert_eq!(count_after_second, 1);
    }

    #[test]
    fn replayed_zero_accrual_posted_is_idempotent() {
        let mut plan = initial_plan_no_structuring_fee();

        let activate_event = CoreCreditEvent::FacilityActivated {
            entity: crate::PublicCreditFacility {
                id: CreditFacilityId::new(),
                customer_id: CustomerId::new(),
                collateral_id: CollateralId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
                completed_at: None,
                liquidation_trigger: None,
                collateralization: default_collateralization(),
            },
        };
        plan.process_credit_event(
            Default::default(),
            &activate_event,
            default_start_date(),
            default_start_date(),
        );

        let period = InterestInterval::EndOfMonth.period_from(default_start_date());
        let accrual_event = accrual_posted_event(period);

        // First processing should create the accrual entry
        let first_result = plan.process_credit_event(
            Default::default(),
            &accrual_event,
            default_start_date(),
            default_start_date(),
        );
        assert!(first_result);

        let count_after_first = plan
            .entries
            .iter()
            .filter(|e| {
                e.repayment_type == RepaymentType::Interest
                    && e.status == RepaymentStatus::Paid
                    && e.obligation_id.is_none()
            })
            .count();
        assert_eq!(count_after_first, 1);

        // Second processing (replay) should be idempotent - no duplicate entry
        let second_result = plan.process_credit_event(
            Default::default(),
            &accrual_event,
            default_start_date(),
            default_start_date(),
        );
        assert!(!second_result);

        let count_after_second = plan
            .entries
            .iter()
            .filter(|e| {
                e.repayment_type == RepaymentType::Interest
                    && e.status == RepaymentStatus::Paid
                    && e.obligation_id.is_none()
            })
            .count();
        assert_eq!(count_after_second, 1);
    }

    #[test]
    fn replayed_facility_proposal_created_is_idempotent() {
        let mut plan = CreditFacilityRepaymentPlan::default();

        let proposal_event = CoreCreditEvent::FacilityProposalCreated {
            entity: crate::PublicCreditFacilityProposal {
                id: CreditFacilityProposalId::new(),
                status: CreditFacilityProposalStatus::PendingCustomerApproval,
                amount: UsdCents::from(100_000_00),
                terms: terms(0),
                customer_id: CustomerId::new(),
                created_at: default_start_date(),
            },
        };

        // First processing
        let first_result = plan.process_credit_event(
            Default::default(),
            &proposal_event,
            default_start_date(),
            default_start_date(),
        );
        assert!(first_result);
        assert_eq!(plan.facility_amount, UsdCents::from(100_000_00));

        // Second processing (replay) - naturally idempotent, sets same values
        let second_result = plan.process_credit_event(
            Default::default(),
            &proposal_event,
            default_start_date(),
            default_start_date(),
        );
        assert!(second_result);
        assert_eq!(plan.facility_amount, UsdCents::from(100_000_00));
    }

    #[test]
    fn replayed_facility_activated_is_idempotent() {
        let mut plan = initial_plan();
        let activated_at = default_start_date();

        let activate_event = CoreCreditEvent::FacilityActivated {
            entity: crate::PublicCreditFacility {
                id: CreditFacilityId::new(),
                customer_id: CustomerId::new(),
                collateral_id: CollateralId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at,
                amount: default_facility_amount(),
                completed_at: None,
                liquidation_trigger: None,
                collateralization: default_collateralization(),
            },
        };

        // First processing
        let first_result = plan.process_credit_event(
            Default::default(),
            &activate_event,
            default_start_date(),
            default_start_date(),
        );
        assert!(first_result);
        assert_eq!(plan.activated_at, Some(activated_at));

        // Second processing (replay) - naturally idempotent, sets same value
        let second_result = plan.process_credit_event(
            Default::default(),
            &activate_event,
            default_start_date(),
            default_start_date(),
        );
        assert!(second_result);
        assert_eq!(plan.activated_at, Some(activated_at));
    }

    #[test]
    fn replayed_obligation_status_changes_are_idempotent() {
        let obligation_id = ObligationId::new();
        let mut plan = initial_plan();
        let recorded_at = default_start_date();
        let obligation_entity = PublicObligation {
            id: obligation_id,
            obligation_type: ObligationType::Disbursal,
            beneficiary_id: CreditFacilityId::new().into(),
            initial_amount: UsdCents::from(100_000_00),
            outstanding_amount: UsdCents::from(100_000_00),
            due_at: EffectiveDate::from(recorded_at),
            overdue_at: None,
            defaulted_at: None,
            recorded_at,
            effective: recorded_at.date_naive(),
        };

        // Setup: activate and create obligation
        let events = vec![
            TestEvent::Credit(CoreCreditEvent::FacilityActivated {
                entity: crate::PublicCreditFacility {
                    id: CreditFacilityId::new(),
                    customer_id: CustomerId::new(),
                    collateral_id: CollateralId::new(),
                    activation_tx_id: LedgerTxId::new(),
                    activated_at: default_start_date(),
                    amount: default_facility_amount(),
                    completed_at: None,
                    liquidation_trigger: None,
                    collateralization: default_collateralization(),
                },
            }),
            TestEvent::Collection(CoreCreditCollectionEvent::ObligationCreated {
                entity: obligation_entity.clone(),
            }),
        ];
        process_test_events(&mut plan, events);

        // Test ObligationDue replay
        let due_event = CoreCreditCollectionEvent::ObligationDue {
            entity: obligation_entity.clone(),
        };
        plan.process_collection_event(Default::default(), &due_event, default_start_date());
        let status_after_first = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status_after_first, RepaymentStatus::Due);

        // Replay - naturally idempotent
        plan.process_collection_event(Default::default(), &due_event, default_start_date());
        let status_after_second = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status_after_second, RepaymentStatus::Due);

        // Test ObligationOverdue replay
        let overdue_event = CoreCreditCollectionEvent::ObligationOverdue {
            entity: obligation_entity.clone(),
        };
        plan.process_collection_event(Default::default(), &overdue_event, default_start_date());
        plan.process_collection_event(Default::default(), &overdue_event, default_start_date());
        let status = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status, RepaymentStatus::Overdue);

        // Test ObligationDefaulted replay
        let defaulted_event = CoreCreditCollectionEvent::ObligationDefaulted {
            entity: obligation_entity.clone(),
        };
        plan.process_collection_event(Default::default(), &defaulted_event, default_start_date());
        plan.process_collection_event(Default::default(), &defaulted_event, default_start_date());
        let status = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status, RepaymentStatus::Defaulted);

        // Test ObligationCompleted replay
        let completed_event = CoreCreditCollectionEvent::ObligationCompleted {
            entity: obligation_entity,
        };
        plan.process_collection_event(Default::default(), &completed_event, default_start_date());
        plan.process_collection_event(Default::default(), &completed_event, default_start_date());
        let status = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status, RepaymentStatus::Paid);
    }
}
