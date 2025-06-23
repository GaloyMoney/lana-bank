use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use lana_events::{CoreCreditEvent, LanaEvent};
use outbox::Outbox;

use crate::email::EmailNotification;

#[derive(Serialize, Deserialize)]
pub struct EmailEventListenerConfig;

impl JobConfig for EmailEventListenerConfig {
    type Initializer = EmailEventListenerInit;
}

pub struct EmailEventListenerInit {
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification,
}

impl EmailEventListenerInit {
    pub fn new(outbox: &Outbox<LanaEvent>, email_notification: &EmailNotification) -> Self {
        Self {
            outbox: outbox.clone(),
            email_notification: email_notification.clone(),
        }
    }
}

const EMAIL_LISTENER_JOB: JobType = JobType::new("email-listener");
impl JobInitializer for EmailEventListenerInit {
    fn job_type() -> JobType {
        EMAIL_LISTENER_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailEventListenerRunner {
            outbox: self.outbox.clone(),
            email_notification: self.email_notification.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Serialize, Deserialize)]
struct EmailEventListenerData {
    sequence: outbox::EventSequence,
}

pub struct EmailEventListenerRunner {
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification,
}

#[async_trait]
impl JobRunner for EmailEventListenerRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<EmailEventListenerData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;
        while let Some(message) = stream.next().await {
            let mut db = es_entity::DbOp::init(current_job.pool()).await?;
            if let Some(event) = &message.payload {
                self.handle_event(&mut db, event).await?;
            }
            state.sequence = message.sequence;
            current_job
                .update_execution_state_in_tx(db.tx(), &state)
                .await?;
            db.commit().await?;
        }
        Ok(JobCompletion::RescheduleNow)
    }
}

impl EmailEventListenerRunner {
    async fn handle_event(
        &self,
        db: &mut es_entity::DbOp<'_>,
        event: &LanaEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let LanaEvent::Credit(CoreCreditEvent::ObligationOverdue {
            id,
            credit_facility_id,
            amount,
        }) = event
        {
            self.email_notification
                .send_obligation_overdue_notification(db, id, credit_facility_id, amount)
                .await?;
        }
        Ok(())
    }
}
