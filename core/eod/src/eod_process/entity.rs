use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use super::error::EodProcessError;
use crate::primitives::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobTerminalState {
    Completed,
    Failed,
    Cancelled,
}

impl From<job::JobTerminalState> for JobTerminalState {
    fn from(jts: job::JobTerminalState) -> Self {
        match jts {
            job::JobTerminalState::Completed => Self::Completed,
            job::JobTerminalState::Errored => Self::Failed,
            job::JobTerminalState::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "EodProcessId")]
pub enum EodProcessEvent {
    Initialized {
        id: EodProcessId,
        date: chrono::NaiveDate,
    },
    #[serde(alias = "phase1_started")]
    ObligationsAndDepositsStarted {
        obligation_job_id: job::JobId,
        deposit_job_id: job::JobId,
    },
    #[serde(alias = "phase1_obligation_completed")]
    ObligationStatusCompleted {
        terminal_state: JobTerminalState,
    },
    #[serde(alias = "phase1_deposit_completed")]
    DepositActivityCompleted {
        terminal_state: JobTerminalState,
    },
    #[serde(alias = "phase2_started")]
    CreditFacilityEodStarted {
        credit_facility_job_id: job::JobId,
    },
    #[serde(alias = "phase2_credit_facility_completed")]
    CreditFacilityEodCompleted {
        terminal_state: JobTerminalState,
    },
    Completed {},
    Failed {
        reason: String,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct EodProcess {
    pub id: EodProcessId,
    pub date: chrono::NaiveDate,
    events: EntityEvents<EodProcessEvent>,
}

impl EodProcess {
    /// Derive status from events via a single reverse scan.
    pub fn status(&self) -> EodProcessStatus {
        let mut has_credit_facility_eod_started = false;
        let mut obligation_done = false;
        let mut deposit_done = false;
        let mut has_obligations_and_deposits_started = false;

        for event in self.events.iter_all().rev() {
            match event {
                EodProcessEvent::Completed { .. } => return EodProcessStatus::Completed,
                EodProcessEvent::Failed { .. } => return EodProcessStatus::Failed,
                EodProcessEvent::CreditFacilityEodStarted { .. } => {
                    has_credit_facility_eod_started = true
                }
                EodProcessEvent::ObligationStatusCompleted { .. } => obligation_done = true,
                EodProcessEvent::DepositActivityCompleted { .. } => deposit_done = true,
                EodProcessEvent::ObligationsAndDepositsStarted { .. } => {
                    has_obligations_and_deposits_started = true
                }
                EodProcessEvent::Initialized { .. }
                | EodProcessEvent::CreditFacilityEodCompleted { .. } => {}
            }
        }

        if has_credit_facility_eod_started {
            EodProcessStatus::AwaitingCreditFacilityEod
        } else if obligation_done && deposit_done {
            EodProcessStatus::ObligationsAndDepositsComplete
        } else if has_obligations_and_deposits_started {
            EodProcessStatus::AwaitingObligationsAndDeposits
        } else {
            EodProcessStatus::Initialized
        }
    }

    /// Returns (obligation_job_id, deposit_job_id) if the obligations-and-deposits
    /// phase has started, or None otherwise.
    pub fn obligations_and_deposits_job_ids(&self) -> Option<(job::JobId, job::JobId)> {
        self.events.iter_all().find_map(|e| match e {
            EodProcessEvent::ObligationsAndDepositsStarted {
                obligation_job_id,
                deposit_job_id,
            } => Some((*obligation_job_id, *deposit_job_id)),
            _ => None,
        })
    }

    pub fn credit_facility_job_id(&self) -> Option<job::JobId> {
        self.events.iter_all().find_map(|e| match e {
            EodProcessEvent::CreditFacilityEodStarted {
                credit_facility_job_id,
                ..
            } => Some(*credit_facility_job_id),
            _ => None,
        })
    }

    // --- Command methods ---

    pub fn start_obligations_and_deposits(
        &mut self,
        obligation_job_id: job::JobId,
        deposit_job_id: job::JobId,
    ) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::ObligationsAndDepositsStarted { .. }
        );
        if self.status() != EodProcessStatus::Initialized {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "start_obligations_and_deposits",
            });
        }
        self.events
            .push(EodProcessEvent::ObligationsAndDepositsStarted {
                obligation_job_id,
                deposit_job_id,
            });
        Ok(Idempotent::Executed(()))
    }

    /// Record the results of the obligations-and-deposits phase.
    /// If both succeeded, the entity transitions to `ObligationsAndDepositsComplete`.
    /// If any failed, the entity transitions to `Failed`.
    pub fn complete_obligations_and_deposits(
        &mut self,
        obligation_terminal: JobTerminalState,
        deposit_terminal: JobTerminalState,
    ) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::ObligationStatusCompleted { .. }
        );
        if self.status() != EodProcessStatus::AwaitingObligationsAndDeposits {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "complete_obligations_and_deposits",
            });
        }
        self.events
            .push(EodProcessEvent::ObligationStatusCompleted {
                terminal_state: obligation_terminal,
            });
        self.events.push(EodProcessEvent::DepositActivityCompleted {
            terminal_state: deposit_terminal,
        });

        if obligation_terminal != JobTerminalState::Completed
            || deposit_terminal != JobTerminalState::Completed
        {
            let reason = format!(
                "Obligations-and-deposits children failed: obligation={:?}, deposit={:?}",
                obligation_terminal, deposit_terminal
            );
            self.events.push(EodProcessEvent::Failed { reason });
        }
        Ok(Idempotent::Executed(()))
    }

    /// Start the credit-facility EOD phase. The caller must spawn the child
    /// job in the same transaction.
    pub fn start_credit_facility_eod(
        &mut self,
        credit_facility_job_id: job::JobId,
    ) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::CreditFacilityEodStarted { .. }
        );
        if self.status() != EodProcessStatus::ObligationsAndDepositsComplete {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "start_credit_facility_eod",
            });
        }
        self.events.push(EodProcessEvent::CreditFacilityEodStarted {
            credit_facility_job_id,
        });
        Ok(Idempotent::Executed(()))
    }

    /// Record the result of the credit-facility-eod phase and finalize:
    /// - If succeeded: mark completed.
    /// - If failed: mark as failed.
    pub fn complete_credit_facility_eod(
        &mut self,
        terminal_state: JobTerminalState,
    ) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Failed { .. }
        );
        if self.status() != EodProcessStatus::AwaitingCreditFacilityEod {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "complete_credit_facility_eod",
            });
        }
        self.events
            .push(EodProcessEvent::CreditFacilityEodCompleted { terminal_state });
        if terminal_state == JobTerminalState::Completed {
            self.events.push(EodProcessEvent::Completed {});
        } else {
            let reason = format!("Credit-facility-eod child failed: {:?}", terminal_state);
            self.events.push(EodProcessEvent::Failed { reason });
        }
        Ok(Idempotent::Executed(()))
    }

    pub fn mark_failed(&mut self, reason: String) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Failed { .. },
            already_applied: EodProcessEvent::Completed { .. }
        );
        match self.status() {
            EodProcessStatus::AwaitingObligationsAndDeposits
            | EodProcessStatus::ObligationsAndDepositsComplete
            | EodProcessStatus::AwaitingCreditFacilityEod => {}
            current => {
                return Err(EodProcessError::InvalidStateTransition {
                    current,
                    attempted: "mark_failed",
                });
            }
        }
        self.events.push(EodProcessEvent::Failed { reason });
        Ok(Idempotent::Executed(()))
    }
}

impl TryFromEvents<EodProcessEvent> for EodProcess {
    fn try_from_events(
        events: EntityEvents<EodProcessEvent>,
    ) -> Result<Self, EntityHydrationError> {
        let mut builder = EodProcessBuilder::default();
        for event in events.iter_all() {
            if let EodProcessEvent::Initialized { id, date, .. } = event {
                builder = builder.id(*id).date(*date);
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewEodProcess {
    #[builder(setter(into))]
    pub(super) id: EodProcessId,
    pub(super) date: chrono::NaiveDate,
}

impl NewEodProcess {
    pub fn builder() -> NewEodProcessBuilder {
        NewEodProcessBuilder::default()
    }

    pub fn status(&self) -> EodProcessStatus {
        EodProcessStatus::Initialized
    }
}

impl IntoEvents<EodProcessEvent> for NewEodProcess {
    fn into_events(self) -> EntityEvents<EodProcessEvent> {
        EntityEvents::init(
            self.id,
            [EodProcessEvent::Initialized {
                id: self.id,
                date: self.date,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_events(date: chrono::NaiveDate) -> EntityEvents<EodProcessEvent> {
        let id = EodProcessId::new();
        EntityEvents::init(id, [EodProcessEvent::Initialized { id, date }])
    }

    #[test]
    fn initial_status_is_initialized() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        assert_eq!(process.status(), EodProcessStatus::Initialized);
    }

    #[test]
    fn start_obligations_and_deposits_is_idempotent() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        assert!(
            process
                .start_obligations_and_deposits(job1, job2)
                .unwrap()
                .did_execute()
        );
        assert!(
            process
                .start_obligations_and_deposits(job1, job2)
                .unwrap()
                .was_already_applied()
        );
        assert_eq!(
            process.status(),
            EodProcessStatus::AwaitingObligationsAndDeposits
        );
    }

    #[test]
    fn start_obligations_and_deposits_rejects_wrong_state() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let cf_job = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let _ = process
            .complete_obligations_and_deposits(
                JobTerminalState::Completed,
                JobTerminalState::Completed,
            )
            .unwrap();
        let _ = process.start_credit_facility_eod(cf_job).unwrap();
        let _ = process
            .complete_credit_facility_eod(JobTerminalState::Completed)
            .unwrap();
        // In Completed state, start_obligations_and_deposits returns AlreadyApplied
        // because ObligationsAndDepositsStarted event exists in history (idempotency guard)
        assert!(
            process
                .start_obligations_and_deposits(job1, job2)
                .unwrap()
                .was_already_applied()
        );
    }

    #[test]
    fn complete_credit_facility_eod_requires_awaiting_state() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        // Cannot complete_credit_facility_eod from Initialized state
        assert!(
            process
                .complete_credit_facility_eod(JobTerminalState::Completed)
                .is_err()
        );
    }

    #[test]
    fn mark_failed_from_awaiting_obligations_and_deposits() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        assert!(
            process
                .mark_failed("test".to_string())
                .unwrap()
                .did_execute()
        );
        assert_eq!(process.status(), EodProcessStatus::Failed);
    }

    #[test]
    fn complete_obligations_and_deposits_advances_on_success() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let result = process
            .complete_obligations_and_deposits(
                JobTerminalState::Completed,
                JobTerminalState::Completed,
            )
            .unwrap();
        assert!(result.did_execute());
        assert_eq!(
            process.status(),
            EodProcessStatus::ObligationsAndDepositsComplete
        );
    }

    #[test]
    fn complete_obligations_and_deposits_fails_on_child_failure() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let result = process
            .complete_obligations_and_deposits(
                JobTerminalState::Failed,
                JobTerminalState::Completed,
            )
            .unwrap();
        assert!(result.did_execute());
        assert_eq!(process.status(), EodProcessStatus::Failed);
    }

    #[test]
    fn start_credit_facility_eod_rejects_wrong_state() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let cf_job = job::JobId::from(uuid::Uuid::new_v4());
        // Cannot start credit facility EOD from Initialized state
        assert!(process.start_credit_facility_eod(cf_job).is_err());
    }

    #[test]
    fn start_credit_facility_eod_is_idempotent() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let cf_job = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let _ = process
            .complete_obligations_and_deposits(
                JobTerminalState::Completed,
                JobTerminalState::Completed,
            )
            .unwrap();
        assert!(
            process
                .start_credit_facility_eod(cf_job)
                .unwrap()
                .did_execute()
        );
        assert!(
            process
                .start_credit_facility_eod(cf_job)
                .unwrap()
                .was_already_applied()
        );
        assert_eq!(
            process.status(),
            EodProcessStatus::AwaitingCreditFacilityEod
        );
    }

    #[test]
    fn complete_credit_facility_eod_succeeds() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let cf_job = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let _ = process
            .complete_obligations_and_deposits(
                JobTerminalState::Completed,
                JobTerminalState::Completed,
            )
            .unwrap();
        let _ = process.start_credit_facility_eod(cf_job).unwrap();
        let result = process
            .complete_credit_facility_eod(JobTerminalState::Completed)
            .unwrap();
        assert!(result.did_execute());
        assert_eq!(process.status(), EodProcessStatus::Completed);
    }

    #[test]
    fn complete_credit_facility_eod_fails_on_child_failure() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let cf_job = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let _ = process
            .complete_obligations_and_deposits(
                JobTerminalState::Completed,
                JobTerminalState::Completed,
            )
            .unwrap();
        let _ = process.start_credit_facility_eod(cf_job).unwrap();
        let result = process
            .complete_credit_facility_eod(JobTerminalState::Failed)
            .unwrap();
        assert!(result.did_execute());
        assert_eq!(process.status(), EodProcessStatus::Failed);
    }
}
