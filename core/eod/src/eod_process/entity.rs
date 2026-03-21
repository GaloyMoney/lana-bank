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
        phase_names: Vec<String>,
    },
    PhaseStarted {
        phase_name: String,
        job_id: job::JobId,
    },
    PhaseCompleted {
        phase_name: String,
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
    pub(super) phase_names: Vec<String>,
    events: EntityEvents<EodProcessEvent>,
}

impl EodProcess {
    /// Derive status from events via a single reverse scan.
    pub fn status(&self) -> EodProcessStatus {
        for event in self.events.iter_all().rev() {
            match event {
                EodProcessEvent::Completed { .. } => return EodProcessStatus::Completed,
                EodProcessEvent::Failed { .. } => return EodProcessStatus::Failed,
                EodProcessEvent::PhaseStarted { .. } | EodProcessEvent::PhaseCompleted { .. } => {
                    return EodProcessStatus::InProgress;
                }
                EodProcessEvent::Initialized { .. } => {}
            }
        }
        EodProcessStatus::Initialized
    }

    /// The name of the current in-flight phase (started but not yet completed).
    pub fn current_phase(&self) -> Option<&str> {
        let mut last_started: Option<&str> = None;
        for event in self.events.iter_all() {
            match event {
                EodProcessEvent::PhaseStarted { phase_name, .. } => {
                    last_started = Some(phase_name.as_str());
                }
                EodProcessEvent::PhaseCompleted { .. } => {
                    last_started = None;
                }
                _ => {}
            }
        }
        last_started
    }

    /// The name of the next phase that has not yet been started.
    pub fn next_phase_name(&self) -> Option<&str> {
        let started: Vec<&str> = self
            .events
            .iter_all()
            .filter_map(|e| match e {
                EodProcessEvent::PhaseStarted { phase_name, .. } => Some(phase_name.as_str()),
                _ => None,
            })
            .collect();
        self.phase_names
            .iter()
            .find(|name| !started.contains(&name.as_str()))
            .map(|s| s.as_str())
    }

    /// Find the job_id for a started phase.
    pub fn phase_job_id(&self, name: &str) -> Option<job::JobId> {
        self.events.iter_all().find_map(|e| match e {
            EodProcessEvent::PhaseStarted {
                phase_name, job_id, ..
            } if phase_name == name => Some(*job_id),
            _ => None,
        })
    }

    /// Total number of registered phases.
    pub fn total_phases(&self) -> usize {
        self.phase_names.len()
    }

    /// Number of completed phases.
    pub fn completed_phases(&self) -> usize {
        self.events
            .iter_all()
            .filter(|e| matches!(e, EodProcessEvent::PhaseCompleted { .. }))
            .count()
    }

    // --- Command methods ---

    /// Start a phase by name.
    pub fn start_phase(
        &mut self,
        phase_name: String,
        job_id: job::JobId,
    ) -> Result<Idempotent<()>, EodProcessError> {
        let pn = phase_name.clone();
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::PhaseStarted { phase_name: n, .. } if *n == pn
        );
        let status = self.status();
        if status != EodProcessStatus::Initialized && status != EodProcessStatus::InProgress {
            return Err(EodProcessError::InvalidStateTransition {
                current: status,
                attempted: "start_phase",
            });
        }
        self.events
            .push(EodProcessEvent::PhaseStarted { phase_name, job_id });
        Ok(Idempotent::Executed(()))
    }

    /// Record phase completion. Auto-fails on bad terminal state, auto-completes when all done.
    pub fn complete_phase(
        &mut self,
        phase_name: String,
        terminal_state: JobTerminalState,
    ) -> Result<Idempotent<()>, EodProcessError> {
        let pn = phase_name.clone();
        idempotency_guard!(
            self.events.iter_all(),
            already_applied: EodProcessEvent::PhaseCompleted { phase_name: n, .. } if *n == pn
        );
        let status = self.status();
        if status != EodProcessStatus::InProgress {
            return Err(EodProcessError::InvalidStateTransition {
                current: status,
                attempted: "complete_phase",
            });
        }
        self.events.push(EodProcessEvent::PhaseCompleted {
            phase_name,
            terminal_state,
        });

        if terminal_state != JobTerminalState::Completed {
            let reason = format!("Phase '{}' child failed: {:?}", pn, terminal_state);
            self.events.push(EodProcessEvent::Failed { reason });
        } else if self.completed_phases() == self.total_phases() {
            self.events.push(EodProcessEvent::Completed {});
        }
        Ok(Idempotent::Executed(()))
    }
}

impl TryFromEvents<EodProcessEvent> for EodProcess {
    fn try_from_events(
        events: EntityEvents<EodProcessEvent>,
    ) -> Result<Self, EntityHydrationError> {
        let mut builder = EodProcessBuilder::default();
        for event in events.iter_all() {
            if let EodProcessEvent::Initialized {
                id,
                date,
                phase_names,
            } = event
            {
                builder = builder.id(*id).date(*date).phase_names(phase_names.clone());
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
    pub(super) phase_names: Vec<String>,
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
                phase_names: self.phase_names,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_events(
        date: chrono::NaiveDate,
        phase_names: Vec<String>,
    ) -> EntityEvents<EodProcessEvent> {
        let id = EodProcessId::new();
        EntityEvents::init(
            id,
            [EodProcessEvent::Initialized {
                id,
                date,
                phase_names,
            }],
        )
    }

    fn default_phases() -> Vec<String> {
        vec![
            "obligation-status".to_string(),
            "deposit-activity".to_string(),
            "credit-facility-eod".to_string(),
        ]
    }

    #[test]
    fn initial_status_is_initialized() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let process = EodProcess::try_from_events(init_events(date, default_phases()))
            .expect("Could not build eod process");
        assert_eq!(process.status(), EodProcessStatus::Initialized);
    }

    #[test]
    fn start_phase_is_idempotent() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process = EodProcess::try_from_events(init_events(date, default_phases()))
            .expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        assert!(
            process
                .start_phase("obligation-status".to_string(), job1)
                .unwrap()
                .did_execute()
        );
        assert!(
            process
                .start_phase("obligation-status".to_string(), job1)
                .unwrap()
                .was_already_applied()
        );
        assert_eq!(process.status(), EodProcessStatus::InProgress);
    }

    #[test]
    fn complete_phase_advances_and_auto_completes() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let phases = default_phases();
        let mut process = EodProcess::try_from_events(init_events(date, phases.clone()))
            .expect("Could not build eod process");

        for phase_name in &phases {
            let job_id = job::JobId::from(uuid::Uuid::new_v4());
            let _ = process.start_phase(phase_name.clone(), job_id).unwrap();
            let _ = process
                .complete_phase(phase_name.clone(), JobTerminalState::Completed)
                .unwrap();
        }
        assert_eq!(process.status(), EodProcessStatus::Completed);
    }

    #[test]
    fn complete_phase_fails_on_child_failure() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process = EodProcess::try_from_events(init_events(date, default_phases()))
            .expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process
            .start_phase("obligation-status".to_string(), job1)
            .unwrap();
        let result = process
            .complete_phase("obligation-status".to_string(), JobTerminalState::Failed)
            .unwrap();
        assert!(result.did_execute());
        assert_eq!(process.status(), EodProcessStatus::Failed);
    }

    #[test]
    fn next_phase_name_tracks_progress() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process = EodProcess::try_from_events(init_events(date, default_phases()))
            .expect("Could not build eod process");
        assert_eq!(process.next_phase_name(), Some("obligation-status"));

        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process
            .start_phase("obligation-status".to_string(), job1)
            .unwrap();
        assert_eq!(process.next_phase_name(), Some("deposit-activity"));

        let _ = process
            .complete_phase("obligation-status".to_string(), JobTerminalState::Completed)
            .unwrap();
        assert_eq!(process.next_phase_name(), Some("deposit-activity"));
    }

    #[test]
    fn current_phase_returns_in_flight_phase() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process = EodProcess::try_from_events(init_events(date, default_phases()))
            .expect("Could not build eod process");
        assert_eq!(process.current_phase(), None);

        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process
            .start_phase("obligation-status".to_string(), job1)
            .unwrap();
        assert_eq!(process.current_phase(), Some("obligation-status"));

        let _ = process
            .complete_phase("obligation-status".to_string(), JobTerminalState::Completed)
            .unwrap();
        assert_eq!(process.current_phase(), None);
    }

    #[test]
    fn complete_phase_rejects_wrong_state() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let process = EodProcess::try_from_events(init_events(date, default_phases()))
            .expect("Could not build eod process");
        // Cannot complete_phase from Initialized state
        assert_eq!(process.status(), EodProcessStatus::Initialized);
        // Need to clone, can't mutate since we'd fail
        let mut process2 = EodProcess::try_from_events(init_events(date, default_phases()))
            .expect("Could not build eod process");
        assert!(
            process2
                .complete_phase("obligation-status".to_string(), JobTerminalState::Completed)
                .is_err()
        );
    }

    #[test]
    fn start_phase_rejects_terminal_state() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let phases = default_phases();
        let mut process = EodProcess::try_from_events(init_events(date, phases.clone()))
            .expect("Could not build eod process");

        // Complete all phases to reach Completed state
        for phase_name in &phases {
            let job_id = job::JobId::from(uuid::Uuid::new_v4());
            let _ = process.start_phase(phase_name.clone(), job_id).unwrap();
            let _ = process
                .complete_phase(phase_name.clone(), JobTerminalState::Completed)
                .unwrap();
        }
        assert_eq!(process.status(), EodProcessStatus::Completed);

        // Starting another phase returns AlreadyApplied or error
        let job_new = job::JobId::from(uuid::Uuid::new_v4());
        // All phases already started, so idempotency guard applies
        assert!(
            process
                .start_phase("obligation-status".to_string(), job_new)
                .unwrap()
                .was_already_applied()
        );
    }

    #[test]
    fn phase_job_id_finds_correct_id() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let mut process = EodProcess::try_from_events(init_events(date, default_phases()))
            .expect("Could not build eod process");
        let job1 = job::JobId::from(uuid::Uuid::new_v4());
        let _ = process
            .start_phase("obligation-status".to_string(), job1)
            .unwrap();
        assert_eq!(process.phase_job_id("obligation-status"), Some(job1));
        assert_eq!(process.phase_job_id("deposit-activity"), None);
    }
}
