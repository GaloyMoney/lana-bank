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

/// Outcome of a phase evaluation within the entity.
pub enum PhaseOutcome {
    AllSucceeded,
    Failed { reason: String },
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "EodProcessId")]
pub enum EodProcessEvent {
    Initialized {
        id: EodProcessId,
        date: chrono::NaiveDate,
    },
    Phase1Started {
        obligation_job_id: job::JobId,
        deposit_job_id: job::JobId,
    },
    Phase1ObligationCompleted {
        terminal_state: JobTerminalState,
    },
    Phase1DepositCompleted {
        terminal_state: JobTerminalState,
    },
    Phase2Started {
        credit_facility_job_id: job::JobId,
    },
    Phase2CreditFacilityCompleted {
        terminal_state: JobTerminalState,
    },
    Completed {},
    Failed {
        reason: String,
    },
    CancellationRequested {},
    Cancelled {},
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
        let mut has_phase2_started = false;
        let mut phase1_obligation_done = false;
        let mut phase1_deposit_done = false;
        let mut has_phase1_started = false;

        for event in self.events.iter_all().rev() {
            match event {
                EodProcessEvent::Completed { .. } => return EodProcessStatus::Completed,
                EodProcessEvent::Cancelled { .. } => return EodProcessStatus::Cancelled,
                EodProcessEvent::Failed { .. } => return EodProcessStatus::Failed,
                EodProcessEvent::Phase2Started { .. } => has_phase2_started = true,
                EodProcessEvent::Phase1ObligationCompleted { .. } => phase1_obligation_done = true,
                EodProcessEvent::Phase1DepositCompleted { .. } => phase1_deposit_done = true,
                EodProcessEvent::Phase1Started { .. } => has_phase1_started = true,
                _ => {}
            }
        }

        if has_phase2_started {
            EodProcessStatus::AwaitingCreditFacilityEod
        } else if phase1_obligation_done && phase1_deposit_done {
            EodProcessStatus::ObligationsAndDepositsComplete
        } else if has_phase1_started {
            EodProcessStatus::AwaitingObligationsAndDeposits
        } else {
            EodProcessStatus::Initialized
        }
    }

    pub fn obligation_job_id(&self) -> Option<job::JobId> {
        self.events.iter_all().find_map(|e| match e {
            EodProcessEvent::Phase1Started {
                obligation_job_id, ..
            } => Some(*obligation_job_id),
            _ => None,
        })
    }

    pub fn deposit_job_id(&self) -> Option<job::JobId> {
        self.events.iter_all().find_map(|e| match e {
            EodProcessEvent::Phase1Started { deposit_job_id, .. } => Some(*deposit_job_id),
            _ => None,
        })
    }

    pub fn credit_facility_job_id(&self) -> Option<job::JobId> {
        self.events.iter_all().find_map(|e| match e {
            EodProcessEvent::Phase2Started {
                credit_facility_job_id,
                ..
            } => Some(*credit_facility_job_id),
            _ => None,
        })
    }

    pub fn phase1_obligation_terminal(&self) -> Option<JobTerminalState> {
        self.events.iter_all().find_map(|e| match e {
            EodProcessEvent::Phase1ObligationCompleted { terminal_state } => Some(*terminal_state),
            _ => None,
        })
    }

    pub fn phase1_deposit_terminal(&self) -> Option<JobTerminalState> {
        self.events.iter_all().find_map(|e| match e {
            EodProcessEvent::Phase1DepositCompleted { terminal_state } => Some(*terminal_state),
            _ => None,
        })
    }

    pub fn phase2_credit_facility_terminal(&self) -> Option<JobTerminalState> {
        self.events.iter_all().find_map(|e| match e {
            EodProcessEvent::Phase2CreditFacilityCompleted { terminal_state } => {
                Some(*terminal_state)
            }
            _ => None,
        })
    }

    pub fn cancellation_requested(&self) -> bool {
        self.events
            .iter_all()
            .any(|e| matches!(e, EodProcessEvent::CancellationRequested { .. }))
    }

    // --- Business-rule evaluation methods (tell, don't ask) ---

    /// Evaluate whether the obligations-and-deposits phase succeeded or failed.
    pub fn evaluate_obligations_and_deposits_outcome(&self) -> PhaseOutcome {
        let obligation_ok = self.phase1_obligation_terminal() == Some(JobTerminalState::Completed);
        let deposit_ok = self.phase1_deposit_terminal() == Some(JobTerminalState::Completed);

        if obligation_ok && deposit_ok {
            PhaseOutcome::AllSucceeded
        } else {
            PhaseOutcome::Failed {
                reason: format!(
                    "Obligations-and-deposits children failed: obligation={:?}, deposit={:?}",
                    self.phase1_obligation_terminal(),
                    self.phase1_deposit_terminal()
                ),
            }
        }
    }

    /// Evaluate whether the credit-facility-eod phase succeeded or failed.
    pub fn evaluate_credit_facility_eod_outcome(&self) -> PhaseOutcome {
        let credit_facility_ok =
            self.phase2_credit_facility_terminal() == Some(JobTerminalState::Completed);

        if credit_facility_ok {
            PhaseOutcome::AllSucceeded
        } else {
            PhaseOutcome::Failed {
                reason: format!(
                    "Credit-facility-eod child failed: {:?}",
                    self.phase2_credit_facility_terminal()
                ),
            }
        }
    }

    // --- Combined evaluate-and-transition commands ---

    /// Evaluate phase 1 outcome and transition: advance to phase 2 or fail.
    /// Returns `true` if advanced (caller must spawn the credit facility job).
    pub fn try_advance_to_phase2(
        &mut self,
        credit_facility_job_id: job::JobId,
    ) -> Result<Idempotent<bool>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase2Started { .. },
            already_applied: EodProcessEvent::Failed { .. }
        );
        if self.status() != EodProcessStatus::ObligationsAndDepositsComplete {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "try_advance_to_phase2",
            });
        }
        let obligation_ok = self.phase1_obligation_terminal() == Some(JobTerminalState::Completed);
        let deposit_ok = self.phase1_deposit_terminal() == Some(JobTerminalState::Completed);

        if obligation_ok && deposit_ok {
            self.events.push(EodProcessEvent::Phase2Started {
                credit_facility_job_id,
            });
            Ok(Idempotent::Executed(true))
        } else {
            let reason = format!(
                "Obligations-and-deposits children failed: obligation={:?}, deposit={:?}",
                self.phase1_obligation_terminal(),
                self.phase1_deposit_terminal()
            );
            self.events.push(EodProcessEvent::Failed { reason });
            Ok(Idempotent::Executed(false))
        }
    }

    /// Evaluate phase 2 outcome and finalize: complete or fail.
    /// Returns `true` if completed successfully.
    pub fn try_complete(&mut self) -> Result<Idempotent<bool>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Failed { .. },
            already_applied: EodProcessEvent::Cancelled { .. }
        );
        if self.status() != EodProcessStatus::AwaitingCreditFacilityEod {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "try_complete",
            });
        }
        let phase2_done = self
            .events
            .iter_all()
            .any(|e| matches!(e, EodProcessEvent::Phase2CreditFacilityCompleted { .. }));
        if !phase2_done {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "try_complete",
            });
        }
        let credit_facility_ok =
            self.phase2_credit_facility_terminal() == Some(JobTerminalState::Completed);

        if credit_facility_ok {
            self.events.push(EodProcessEvent::Completed {});
            Ok(Idempotent::Executed(true))
        } else {
            let reason = format!(
                "Credit-facility-eod child failed: {:?}",
                self.phase2_credit_facility_terminal()
            );
            self.events.push(EodProcessEvent::Failed { reason });
            Ok(Idempotent::Executed(false))
        }
    }

    // --- Command methods ---

    pub fn start_obligations_and_deposits(
        &mut self,
        obligation_job_id: job::JobId,
        deposit_job_id: job::JobId,
    ) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase1Started { .. }
        );
        if self.status() != EodProcessStatus::Initialized {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "start_obligations_and_deposits",
            });
        }
        self.events.push(EodProcessEvent::Phase1Started {
            obligation_job_id,
            deposit_job_id,
        });
        Ok(Idempotent::Executed(()))
    }

    pub fn complete_phase1_obligation(
        &mut self,
        terminal_state: JobTerminalState,
    ) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase1ObligationCompleted { .. }
        );
        if self.status() != EodProcessStatus::AwaitingObligationsAndDeposits {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "complete_phase1_obligation",
            });
        }
        self.events
            .push(EodProcessEvent::Phase1ObligationCompleted { terminal_state });
        Ok(Idempotent::Executed(()))
    }

    pub fn complete_phase1_deposit(
        &mut self,
        terminal_state: JobTerminalState,
    ) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase1DepositCompleted { .. }
        );
        if self.status() != EodProcessStatus::AwaitingObligationsAndDeposits {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "complete_phase1_deposit",
            });
        }
        self.events
            .push(EodProcessEvent::Phase1DepositCompleted { terminal_state });
        Ok(Idempotent::Executed(()))
    }

    pub fn start_credit_facility_eod(
        &mut self,
        credit_facility_job_id: job::JobId,
    ) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase2Started { .. }
        );
        if self.status() != EodProcessStatus::ObligationsAndDepositsComplete {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "start_credit_facility_eod",
            });
        }
        self.events.push(EodProcessEvent::Phase2Started {
            credit_facility_job_id,
        });
        Ok(Idempotent::Executed(()))
    }

    pub fn complete_phase2_credit_facility(
        &mut self,
        terminal_state: JobTerminalState,
    ) -> Result<Idempotent<()>, EodProcessError> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase2CreditFacilityCompleted { .. }
        );
        if self.status() != EodProcessStatus::AwaitingCreditFacilityEod {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "complete_phase2_credit_facility",
            });
        }
        self.events
            .push(EodProcessEvent::Phase2CreditFacilityCompleted { terminal_state });
        Ok(Idempotent::Executed(()))
    }

    pub fn mark_completed(&mut self) -> Result<Idempotent<()>, EodProcessError> {
        match self.status() {
            EodProcessStatus::AwaitingCreditFacilityEod => {}
            current => {
                return Err(EodProcessError::InvalidStateTransition {
                    current,
                    attempted: "mark_completed",
                });
            }
        }
        // Verify phase 2 actually completed before allowing completion
        let phase2_done = self
            .events
            .iter_all()
            .any(|e| matches!(e, EodProcessEvent::Phase2CreditFacilityCompleted { .. }));
        if !phase2_done {
            return Err(EodProcessError::InvalidStateTransition {
                current: self.status(),
                attempted: "mark_completed",
            });
        }
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Failed { .. },
            already_applied: EodProcessEvent::Cancelled { .. }
        );
        self.events.push(EodProcessEvent::Completed {});
        Ok(Idempotent::Executed(()))
    }

    pub fn mark_failed(&mut self, reason: String) -> Result<Idempotent<()>, EodProcessError> {
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
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Failed { .. },
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Cancelled { .. }
        );
        self.events.push(EodProcessEvent::Failed { reason });
        Ok(Idempotent::Executed(()))
    }

    pub fn request_cancellation(&mut self) -> Result<Idempotent<()>, EodProcessError> {
        match self.status() {
            EodProcessStatus::Completed
            | EodProcessStatus::Failed
            | EodProcessStatus::Cancelled => {
                return Err(EodProcessError::InvalidStateTransition {
                    current: self.status(),
                    attempted: "request_cancellation",
                });
            }
            _ => {}
        }
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::CancellationRequested { .. },
            already_applied: EodProcessEvent::Cancelled { .. },
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Failed { .. }
        );
        self.events.push(EodProcessEvent::CancellationRequested {});
        Ok(Idempotent::Executed(()))
    }

    pub fn mark_cancelled(&mut self) -> Result<Idempotent<()>, EodProcessError> {
        match self.status() {
            EodProcessStatus::Completed
            | EodProcessStatus::Failed
            | EodProcessStatus::Cancelled => {
                return Err(EodProcessError::InvalidStateTransition {
                    current: self.status(),
                    attempted: "mark_cancelled",
                });
            }
            _ => {}
        }
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Cancelled { .. },
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Failed { .. }
        );
        self.events.push(EodProcessEvent::Cancelled {});
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
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let _ = process
            .complete_phase1_obligation(JobTerminalState::Completed)
            .unwrap();
        let _ = process
            .complete_phase1_deposit(JobTerminalState::Completed)
            .unwrap();
        let job3 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_credit_facility_eod(job3).unwrap();
        let _ = process
            .complete_phase2_credit_facility(JobTerminalState::Completed)
            .unwrap();
        let _ = process.mark_completed().unwrap();
        // In Completed state, start_obligations_and_deposits returns AlreadyApplied
        // because Phase1Started event exists in history (idempotency guard)
        assert!(
            process
                .start_obligations_and_deposits(job1, job2)
                .unwrap()
                .was_already_applied()
        );
    }

    #[test]
    fn mark_completed_requires_awaiting_credit_facility_eod() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        // Cannot mark_completed from Initialized state
        assert!(process.mark_completed().is_err());
    }

    #[test]
    fn mark_completed_requires_phase2_credit_facility_completed() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let _ = process
            .complete_phase1_obligation(JobTerminalState::Completed)
            .unwrap();
        let _ = process
            .complete_phase1_deposit(JobTerminalState::Completed)
            .unwrap();
        let job3 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_credit_facility_eod(job3).unwrap();
        // AwaitingCreditFacilityEod but Phase2CreditFacilityCompleted not yet recorded
        assert!(process.mark_completed().is_err());
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
    fn mark_failed_from_obligations_and_deposits_complete() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let _ = process
            .complete_phase1_obligation(JobTerminalState::Failed)
            .unwrap();
        let _ = process
            .complete_phase1_deposit(JobTerminalState::Completed)
            .unwrap();
        assert_eq!(
            process.status(),
            EodProcessStatus::ObligationsAndDepositsComplete
        );
        assert!(
            process
                .mark_failed("obligation failed".to_string())
                .unwrap()
                .did_execute()
        );
        assert_eq!(process.status(), EodProcessStatus::Failed);
    }

    #[test]
    fn evaluate_obligations_and_deposits_outcome_success() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let _ = process
            .complete_phase1_obligation(JobTerminalState::Completed)
            .unwrap();
        let _ = process
            .complete_phase1_deposit(JobTerminalState::Completed)
            .unwrap();
        assert!(matches!(
            process.evaluate_obligations_and_deposits_outcome(),
            PhaseOutcome::AllSucceeded
        ));
    }

    #[test]
    fn evaluate_obligations_and_deposits_outcome_failure() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process.start_obligations_and_deposits(job1, job2).unwrap();
        let _ = process
            .complete_phase1_obligation(JobTerminalState::Failed)
            .unwrap();
        let _ = process
            .complete_phase1_deposit(JobTerminalState::Completed)
            .unwrap();
        assert!(matches!(
            process.evaluate_obligations_and_deposits_outcome(),
            PhaseOutcome::Failed { .. }
        ));
    }
}
