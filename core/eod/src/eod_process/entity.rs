use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobTerminalState {
    Completed,
    Failed,
    Cancelled,
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
        phase: u8,
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
    /// Derive status from events by scanning backwards for terminal states,
    /// then forwards for progress markers.
    pub fn status(&self) -> EodProcessStatus {
        // Check terminal states first (scan backwards)
        for event in self.events.iter_all().rev() {
            match event {
                EodProcessEvent::Completed { .. } => return EodProcessStatus::Completed,
                EodProcessEvent::Cancelled { .. } => return EodProcessStatus::Cancelled,
                EodProcessEvent::Failed { .. } => return EodProcessStatus::Failed,
                _ => {}
            }
        }

        // Derive from progress events
        let has_phase2_started = self
            .events
            .iter_all()
            .any(|e| matches!(e, EodProcessEvent::Phase2Started { .. }));
        let has_phase1_completed =
            self.phase1_obligation_terminal().is_some() && self.phase1_deposit_terminal().is_some();
        let has_phase1_started = self
            .events
            .iter_all()
            .any(|e| matches!(e, EodProcessEvent::Phase1Started { .. }));

        if has_phase2_started {
            EodProcessStatus::AwaitingPhase2
        } else if has_phase1_completed {
            EodProcessStatus::Phase1Complete
        } else if has_phase1_started {
            EodProcessStatus::AwaitingPhase1
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

    // --- Command methods ---

    pub fn start_phase1(
        &mut self,
        obligation_job_id: job::JobId,
        deposit_job_id: job::JobId,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase1Started { .. }
        );
        self.events.push(EodProcessEvent::Phase1Started {
            obligation_job_id,
            deposit_job_id,
        });
        Idempotent::Executed(())
    }

    pub fn complete_phase1_obligation(
        &mut self,
        terminal_state: JobTerminalState,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase1ObligationCompleted { .. }
        );
        self.events
            .push(EodProcessEvent::Phase1ObligationCompleted { terminal_state });
        Idempotent::Executed(())
    }

    pub fn complete_phase1_deposit(&mut self, terminal_state: JobTerminalState) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase1DepositCompleted { .. }
        );
        self.events
            .push(EodProcessEvent::Phase1DepositCompleted { terminal_state });
        Idempotent::Executed(())
    }

    pub fn start_phase2(&mut self, credit_facility_job_id: job::JobId) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase2Started { .. }
        );
        self.events.push(EodProcessEvent::Phase2Started {
            credit_facility_job_id,
        });
        Idempotent::Executed(())
    }

    pub fn complete_phase2_credit_facility(
        &mut self,
        terminal_state: JobTerminalState,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Phase2CreditFacilityCompleted { .. }
        );
        self.events
            .push(EodProcessEvent::Phase2CreditFacilityCompleted { terminal_state });
        Idempotent::Executed(())
    }

    pub fn mark_completed(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Failed { .. },
            already_applied: EodProcessEvent::Cancelled { .. }
        );
        self.events.push(EodProcessEvent::Completed {});
        Idempotent::Executed(())
    }

    pub fn mark_failed(&mut self, phase: u8, reason: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Failed { .. },
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Cancelled { .. }
        );
        self.events.push(EodProcessEvent::Failed { phase, reason });
        Idempotent::Executed(())
    }

    pub fn request_cancellation(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::CancellationRequested { .. },
            already_applied: EodProcessEvent::Cancelled { .. },
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Failed { .. }
        );
        self.events.push(EodProcessEvent::CancellationRequested {});
        Idempotent::Executed(())
    }

    pub fn mark_cancelled(&mut self) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::Cancelled { .. },
            already_applied: EodProcessEvent::Completed { .. },
            already_applied: EodProcessEvent::Failed { .. }
        );
        self.events.push(EodProcessEvent::Cancelled {});
        Idempotent::Executed(())
    }
}

impl TryFromEvents<EodProcessEvent> for EodProcess {
    fn try_from_events(
        events: EntityEvents<EodProcessEvent>,
    ) -> Result<Self, EntityHydrationError> {
        let mut builder = EodProcessBuilder::default();
        for event in events.iter_all() {
            match event {
                EodProcessEvent::Initialized { id, date, .. } => {
                    builder = builder.id(*id).date(*date);
                }
                _ => {}
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
    fn start_phase1_is_idempotent() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let job2 = job::JobId::from(uuid::Uuid::new_v4());
        assert!(process.start_phase1(job1, job2).did_execute());
        assert!(process.start_phase1(job1, job2).was_already_applied());
        assert_eq!(process.status(), EodProcessStatus::AwaitingPhase1);
    }

    #[test]
    fn mark_completed_is_idempotent() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        assert!(process.mark_completed().did_execute());
        assert!(process.mark_completed().was_already_applied());
        assert_eq!(process.status(), EodProcessStatus::Completed);
    }

    #[test]
    fn mark_failed_blocks_completed() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process =
            EodProcess::try_from_events(init_events(date)).expect("Could not build eod process");
        assert!(process.mark_failed(1, "test".to_string()).did_execute());
        assert!(process.mark_completed().was_already_applied());
        assert_eq!(process.status(), EodProcessStatus::Failed);
    }
}
