use serde::{Serialize, de::DeserializeOwned};
use sqlx::{PgPool, Postgres, Transaction};

use super::{JobId, error::JobError};

pub struct CurrentJob {
    id: JobId,
    attempt: u32,
    pool: PgPool,
    execution_state_json: Option<serde_json::Value>,
}

impl CurrentJob {
    pub(super) fn new(
        id: JobId,
        attempt: u32,
        pool: PgPool,
        execution_state: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id,
            attempt,
            pool,
            execution_state_json: execution_state,
        }
    }

    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    pub fn execution_state<T: DeserializeOwned>(&self) -> Result<Option<T>, serde_json::Error> {
        if let Some(execution_state) = self.execution_state_json.as_ref() {
            serde_json::from_value(execution_state.clone()).map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn update_execution_state_in_tx<T: Serialize>(
        &mut self,
        db: &mut Transaction<'_, Postgres>,
        execution_state: &T,
    ) -> Result<(), JobError> {
        let execution_state_json = serde_json::to_value(execution_state)
            .map_err(JobError::CouldNotSerializeExecutionState)?;
        sqlx::query!(
            r#"
          UPDATE job_executions
          SET execution_state_json = $1
          WHERE id = $2
        "#,
            execution_state_json,
            &self.id as &JobId
        )
        .execute(&mut **db)
        .await?;
        self.execution_state_json = Some(execution_state_json);
        Ok(())
    }

    pub async fn update_execution_state<T: Serialize>(
        &mut self,
        execution_state: T,
    ) -> Result<(), JobError> {
        let execution_state_json = serde_json::to_value(execution_state)
            .map_err(JobError::CouldNotSerializeExecutionState)?;
        sqlx::query!(
            r#"
          UPDATE job_executions
          SET execution_state_json = $1
          WHERE id = $2
        "#,
            execution_state_json,
            &self.id as &JobId
        )
        .execute(&self.pool)
        .await?;
        self.execution_state_json = Some(execution_state_json);
        Ok(())
    }

    pub fn id(&self) -> &JobId {
        &self.id
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
