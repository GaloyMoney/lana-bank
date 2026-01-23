mod entry;
pub mod error;
mod jobs;
mod repo;

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker};

use crate::{event::CoreCreditEvent, primitives::*, terms::TermValues};
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

    pub(crate) fn process_event(
        &mut self,
        sequence: EventSequence,
        event: &CoreCreditEvent,
        now: DateTime<Utc>,
    ) -> bool {
        self.last_updated_on_sequence = sequence;

        let mut existing_obligations = self.existing_obligations();

        match event {
            CoreCreditEvent::FacilityProposalCreated { terms, amount, .. } => {
                self.terms = Some(*terms);
                self.facility_amount = *amount;
            }
            CoreCreditEvent::FacilityActivated { activated_at, .. } => {
                self.activated_at = Some(*activated_at);
            }
            CoreCreditEvent::ObligationCreated {
                id,
                obligation_type,
                amount,
                due_at,
                overdue_at,
                defaulted_at,
                recorded_at,
                effective,
                ..
            } => {
                // Skip if already processed (idempotent for replay)
                if existing_obligations
                    .iter()
                    .any(|e| e.obligation_id == Some(*id))
                {
                    return false;
                }

                let entry = CreditFacilityRepaymentPlanEntry {
                    repayment_type: obligation_type.into(),
                    obligation_id: Some(*id),
                    status: RepaymentStatus::NotYetDue,

                    initial: *amount,
                    outstanding: *amount,

                    due_at: *due_at,
                    overdue_at: overdue_at.map(EffectiveDate::from),
                    defaulted_at: defaulted_at.map(EffectiveDate::from),
                    recorded_at: *recorded_at,
                    effective: *effective,
                };
                if *obligation_type == ObligationType::Interest {
                    let effective = EffectiveDate::from(*effective);
                    self.last_interest_accrual_at = Some(effective.end_of_day());
                }

                existing_obligations.push(entry);
            }
            CoreCreditEvent::AccrualPosted {
                ledger_tx_id,
                amount,
                due_at,
                effective,
                recorded_at,
                ..
            } if amount.is_zero() => {
                // Skip if already processed (idempotent for replay)
                if !self.applied_accruals.insert(*ledger_tx_id) {
                    return false;
                }

                let entry = CreditFacilityRepaymentPlanEntry {
                    repayment_type: RepaymentType::Interest,
                    obligation_id: None,
                    status: RepaymentStatus::Paid,

                    initial: UsdCents::ZERO,
                    outstanding: UsdCents::ZERO,

                    due_at: *due_at,
                    overdue_at: None,
                    defaulted_at: None,
                    recorded_at: *recorded_at,
                    effective: *effective,
                };

                let effective = EffectiveDate::from(*effective);
                self.last_interest_accrual_at = Some(effective.end_of_day());

                existing_obligations.push(entry);
            }
            CoreCreditEvent::FacilityPaymentAllocated {
                obligation_id,
                allocation_id,
                amount,
                ..
            } => {
                if !self.applied_allocations.insert(*allocation_id) {
                    return false;
                }

                if let Some(entry) = existing_obligations.iter_mut().find_map(|entry| {
                    (entry.obligation_id == Some(*obligation_id)).then_some(entry)
                }) {
                    entry.outstanding -= *amount;
                } else {
                    return false;
                }
            }
            CoreCreditEvent::ObligationDue {
                id: obligation_id, ..
            }
            | CoreCreditEvent::ObligationOverdue {
                id: obligation_id, ..
            }
            | CoreCreditEvent::ObligationDefaulted {
                id: obligation_id, ..
            }
            | CoreCreditEvent::ObligationCompleted {
                id: obligation_id, ..
            } => {
                if let Some(entry) = existing_obligations.iter_mut().find_map(|entry| {
                    (entry.obligation_id == Some(*obligation_id)).then_some(entry)
                }) {
                    entry.status = match event {
                        CoreCreditEvent::ObligationDue { .. } => RepaymentStatus::Due,
                        CoreCreditEvent::ObligationOverdue { .. } => RepaymentStatus::Overdue,
                        CoreCreditEvent::ObligationDefaulted { .. } => RepaymentStatus::Defaulted,
                        CoreCreditEvent::ObligationCompleted { .. } => RepaymentStatus::Paid,
                        _ => unreachable!(),
                    };
                } else {
                    return false;
                }
            }

            _ => return false,
        };

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

        true
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
        E: OutboxEventMarker<CoreCreditEvent>,
    {
        let repo = Arc::new(RepaymentPlanRepo::new(pool));

        let job_init =
            credit_facility_repayment_plan::RepaymentPlanProjectionInit::new(outbox, repo.clone());

        let spawner = jobs.add_initializer(job_init);

        spawner
            .spawn_unique(
                job::JobId::new(),
                credit_facility_repayment_plan::RepaymentPlanProjectionConfig {
                    _phantom: std::marker::PhantomData,
                },
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

    use crate::terms::{
        DisbursalPolicy, FacilityDuration, InterestInterval, ObligationDuration, OneTimeFeeRatePct,
    };

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

    fn plan(terms: TermValues) -> CreditFacilityRepaymentPlan {
        let mut plan = CreditFacilityRepaymentPlan::default();
        plan.process_event(
            Default::default(),
            &CoreCreditEvent::FacilityProposalCreated {
                id: CreditFacilityProposalId::new(),
                terms,
                amount: default_facility_amount(),
                created_at: default_start_date(),
            },
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

    fn process_events(plan: &mut CreditFacilityRepaymentPlan, events: Vec<CoreCreditEvent>) {
        for event in events {
            plan.process_event(Default::default(), &event, default_start_date());
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
            id: CreditFacilityId::new(),
            activation_tx_id: LedgerTxId::new(),
            activated_at: default_start_date(),
            amount: default_facility_amount(),
        }];
        process_events(&mut plan, events);

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
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::AccrualPosted {
                credit_facility_id: CreditFacilityId::new(),
                ledger_tx_id: LedgerTxId::new(),
                amount: UsdCents::ZERO,
                period,
                due_at: EffectiveDate::from(period.end),
                recorded_at: period.end,
                effective: period.end.date_naive(),
            },
        ];
        process_events(&mut plan, events);

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
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::AccrualPosted {
                credit_facility_id: CreditFacilityId::new(),
                ledger_tx_id: LedgerTxId::new(),
                amount: UsdCents::ZERO,
                period: period_1,
                due_at: EffectiveDate::from(period_1.end),
                recorded_at: period_1.end,
                effective: period_1.end.date_naive(),
            },
            CoreCreditEvent::AccrualPosted {
                credit_facility_id: CreditFacilityId::new(),
                ledger_tx_id: LedgerTxId::new(),
                amount: UsdCents::ZERO,
                period: period_2,
                due_at: EffectiveDate::from(period_2.end),
                recorded_at: period_2.end,
                effective: period_2.end.date_naive(),
            },
        ];
        process_events(&mut plan, events);

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
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: EffectiveDate::from(default_start_date()),
                overdue_at: None,
                defaulted_at: None,
                recorded_at,
                effective: recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

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
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: EffectiveDate::from(disbursal_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: disbursal_recorded_at,
                effective: disbursal_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: EffectiveDate::from(interest_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

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
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: EffectiveDate::from(disbursal_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: disbursal_recorded_at,
                effective: disbursal_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: EffectiveDate::from(interest_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
            CoreCreditEvent::FacilityPaymentAllocated {
                credit_facility_id: CreditFacilityId::new(),
                obligation_id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                allocation_id: PaymentAllocationId::new(),
                amount: UsdCents::from(400_00),
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

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
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: EffectiveDate::from(disbursal_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: disbursal_recorded_at,
                effective: disbursal_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: EffectiveDate::from(interest_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
            CoreCreditEvent::FacilityPaymentAllocated {
                credit_facility_id: CreditFacilityId::new(),
                obligation_id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                allocation_id: PaymentAllocationId::new(),
                amount: UsdCents::from(1_000_00),
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

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

        plan.process_event(
            Default::default(),
            &CoreCreditEvent::ObligationCompleted {
                id: interest_obligation_id,
                credit_facility_id: CreditFacilityId::new(),
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
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: EffectiveDate::from(disbursal_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: disbursal_recorded_at,
                effective: disbursal_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: EffectiveDate::from(interest_1_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: interest_1_recorded_at,
                effective: interest_1_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: EffectiveDate::from(interest_2_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: interest_2_recorded_at,
                effective: interest_2_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: EffectiveDate::from(interest_3_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: interest_3_recorded_at,
                effective: interest_3_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(33_00),
                due_at: EffectiveDate::from(interest_4_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: interest_4_recorded_at,
                effective: interest_4_recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

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
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: EffectiveDate::from(disbursal_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: disbursal_recorded_at,
                effective: disbursal_recorded_at.date_naive(),
            },
            CoreCreditEvent::ObligationCreated {
                id: interest_obligation_id,
                obligation_type: ObligationType::Interest,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(1_000_00),
                due_at: EffectiveDate::from(interest_recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at: interest_recorded_at,
                effective: interest_recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

        let payment_event = CoreCreditEvent::FacilityPaymentAllocated {
            credit_facility_id: CreditFacilityId::new(),
            obligation_id: interest_obligation_id,
            obligation_type: ObligationType::Interest,
            allocation_id,
            amount: UsdCents::from(1_000_00),
            recorded_at: interest_recorded_at,
            effective: interest_recorded_at.date_naive(),
        };

        // First processing should apply the payment
        let first_result =
            plan.process_event(Default::default(), &payment_event, default_start_date());
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
            plan.process_event(Default::default(), &payment_event, default_start_date());
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
            id: CreditFacilityId::new(),
            activation_tx_id: LedgerTxId::new(),
            activated_at: default_start_date(),
            amount: default_facility_amount(),
        };
        plan.process_event(Default::default(), &activate_event, default_start_date());

        let obligation_event = CoreCreditEvent::ObligationCreated {
            id: obligation_id,
            obligation_type: ObligationType::Disbursal,
            credit_facility_id: CreditFacilityId::new(),
            amount: UsdCents::from(100_000_00),
            due_at: EffectiveDate::from(recorded_at),
            overdue_at: None,
            defaulted_at: None,
            recorded_at,
            effective: recorded_at.date_naive(),
        };

        // First processing should create the obligation entry
        let first_result =
            plan.process_event(Default::default(), &obligation_event, default_start_date());
        assert!(first_result);

        let count_after_first = plan
            .entries
            .iter()
            .filter(|e| e.obligation_id == Some(obligation_id))
            .count();
        assert_eq!(count_after_first, 1);

        // Second processing (replay) should be idempotent - no duplicate entry
        let second_result =
            plan.process_event(Default::default(), &obligation_event, default_start_date());
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
            id: CreditFacilityId::new(),
            activation_tx_id: LedgerTxId::new(),
            activated_at: default_start_date(),
            amount: default_facility_amount(),
        };
        plan.process_event(Default::default(), &activate_event, default_start_date());

        let period = InterestInterval::EndOfMonth.period_from(default_start_date());
        let accrual_event = CoreCreditEvent::AccrualPosted {
            credit_facility_id: CreditFacilityId::new(),
            ledger_tx_id: LedgerTxId::new(),
            amount: UsdCents::ZERO,
            period,
            due_at: EffectiveDate::from(period.end),
            recorded_at: period.end,
            effective: period.end.date_naive(),
        };

        // First processing should create the accrual entry
        let first_result =
            plan.process_event(Default::default(), &accrual_event, default_start_date());
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
        let second_result =
            plan.process_event(Default::default(), &accrual_event, default_start_date());
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
            id: CreditFacilityProposalId::new(),
            amount: UsdCents::from(100_000_00),
            terms: terms(0),
            created_at: default_start_date(),
        };

        // First processing
        let first_result =
            plan.process_event(Default::default(), &proposal_event, default_start_date());
        assert!(first_result);
        assert_eq!(plan.facility_amount, UsdCents::from(100_000_00));

        // Second processing (replay) - naturally idempotent, sets same values
        let second_result =
            plan.process_event(Default::default(), &proposal_event, default_start_date());
        assert!(second_result);
        assert_eq!(plan.facility_amount, UsdCents::from(100_000_00));
    }

    #[test]
    fn replayed_facility_activated_is_idempotent() {
        let mut plan = initial_plan();
        let activated_at = default_start_date();

        let activate_event = CoreCreditEvent::FacilityActivated {
            id: CreditFacilityId::new(),
            activation_tx_id: LedgerTxId::new(),
            activated_at,
            amount: default_facility_amount(),
        };

        // First processing
        let first_result =
            plan.process_event(Default::default(), &activate_event, default_start_date());
        assert!(first_result);
        assert_eq!(plan.activated_at, Some(activated_at));

        // Second processing (replay) - naturally idempotent, sets same value
        let second_result =
            plan.process_event(Default::default(), &activate_event, default_start_date());
        assert!(second_result);
        assert_eq!(plan.activated_at, Some(activated_at));
    }

    #[test]
    fn replayed_obligation_status_changes_are_idempotent() {
        let obligation_id = ObligationId::new();
        let mut plan = initial_plan();
        let recorded_at = default_start_date();

        // Setup: activate and create obligation
        let events = vec![
            CoreCreditEvent::FacilityActivated {
                id: CreditFacilityId::new(),
                activation_tx_id: LedgerTxId::new(),
                activated_at: default_start_date(),
                amount: default_facility_amount(),
            },
            CoreCreditEvent::ObligationCreated {
                id: obligation_id,
                obligation_type: ObligationType::Disbursal,
                credit_facility_id: CreditFacilityId::new(),
                amount: UsdCents::from(100_000_00),
                due_at: EffectiveDate::from(recorded_at),
                overdue_at: None,
                defaulted_at: None,
                recorded_at,
                effective: recorded_at.date_naive(),
            },
        ];
        process_events(&mut plan, events);

        // Test ObligationDue replay
        let due_event = CoreCreditEvent::ObligationDue {
            id: obligation_id,
            credit_facility_id: CreditFacilityId::new(),
            obligation_type: ObligationType::Disbursal,
            amount: UsdCents::from(100_000_00),
        };
        plan.process_event(Default::default(), &due_event, default_start_date());
        let status_after_first = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status_after_first, RepaymentStatus::Due);

        // Replay - naturally idempotent
        plan.process_event(Default::default(), &due_event, default_start_date());
        let status_after_second = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status_after_second, RepaymentStatus::Due);

        // Test ObligationOverdue replay
        let overdue_event = CoreCreditEvent::ObligationOverdue {
            id: obligation_id,
            credit_facility_id: CreditFacilityId::new(),
            amount: UsdCents::from(100_000_00),
        };
        plan.process_event(Default::default(), &overdue_event, default_start_date());
        plan.process_event(Default::default(), &overdue_event, default_start_date());
        let status = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status, RepaymentStatus::Overdue);

        // Test ObligationDefaulted replay
        let defaulted_event = CoreCreditEvent::ObligationDefaulted {
            id: obligation_id,
            credit_facility_id: CreditFacilityId::new(),
            amount: UsdCents::from(100_000_00),
        };
        plan.process_event(Default::default(), &defaulted_event, default_start_date());
        plan.process_event(Default::default(), &defaulted_event, default_start_date());
        let status = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status, RepaymentStatus::Defaulted);

        // Test ObligationCompleted replay
        let completed_event = CoreCreditEvent::ObligationCompleted {
            id: obligation_id,
            credit_facility_id: CreditFacilityId::new(),
        };
        plan.process_event(Default::default(), &completed_event, default_start_date());
        plan.process_event(Default::default(), &completed_event, default_start_date());
        let status = plan
            .entries
            .iter()
            .find(|e| e.obligation_id == Some(obligation_id))
            .unwrap()
            .status;
        assert_eq!(status, RepaymentStatus::Paid);
    }
}
