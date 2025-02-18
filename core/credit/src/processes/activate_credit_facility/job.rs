use async_trait::async_trait;
use futures::StreamExt;

use job::*;
use lana_events::{CreditEvent, LanaEvent};

use super::ActivateCreditFacility;
use crate::outbox::Outbox;

#[derive(serde::Serialize)]
pub(in crate::credit_facility) struct CreditFacilityActivationJobConfig;
impl JobConfig for CreditFacilityActivationJobConfig {
    type Initializer = CreditFacilityActivationJobInitializer;
}

pub(in crate::credit_facility) struct CreditFacilityActivationJobInitializer {
    outbox: Outbox,
    process: ActivateCreditFacility,
}

impl CreditFacilityActivationJobInitializer {
    pub fn new(outbox: &Outbox, process: &ActivateCreditFacility) -> Self {
        Self {
            process: process.clone(),
            outbox: outbox.clone(),
        }
    }
}

const CREDIT_FACILITY_ACTIVATE_JOB: JobType = JobType::new("credit-facility-activation");
impl JobInitializer for CreditFacilityActivationJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_ACTIVATE_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityActivationJobRunner {
            outbox: self.outbox.clone(),
            process: self.process.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct CreditFacilityActivationJobData {
    sequence: outbox::EventSequence,
}

pub struct CreditFacilityActivationJobRunner {
    outbox: Outbox,
    process: ActivateCreditFacility,
}
#[async_trait]
impl JobRunner for CreditFacilityActivationJobRunner {
    #[allow(clippy::single_match)]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityActivationJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.payload {
                Some(LanaEvent::Credit(CreditEvent::FacilityCollateralUpdated { id, .. }))
                | Some(LanaEvent::Credit(CreditEvent::FacilityApproved { id, .. })) => {
                    self.process.execute(id).await?;
                    state.sequence = message.sequence;
                    current_job.update_execution_state(state).await?;
                }
                _ => (),
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
