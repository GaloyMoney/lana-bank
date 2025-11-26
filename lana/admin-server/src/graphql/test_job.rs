use async_graphql::{Context, InputObject, Object, SimpleObject};

use lana_app::{app::LanaApp, test_job::TestWaitJobConfig};

use crate::primitives::*;

#[derive(InputObject)]
pub struct TestJobSpawnInput {
    /// How long the job should wait (in seconds)
    #[graphql(default = 30)]
    pub wait_seconds: u64,
    /// Whether the job should respect shutdown signals
    #[graphql(default = true)]
    pub respect_shutdown: bool,
}

#[derive(SimpleObject)]
pub struct TestJobSpawnPayload {
    pub job_id: UUID,
    pub wait_seconds: u64,
    pub respect_shutdown: bool,
}

#[derive(Default)]
pub struct TestJobMutation;

#[Object]
impl TestJobMutation {
    /// Spawn a test wait job to test graceful shutdown behavior
    pub async fn test_job_spawn(
        &self,
        ctx: &Context<'_>,
        input: TestJobSpawnInput,
    ) -> async_graphql::Result<TestJobSpawnPayload> {
        let app = ctx.data_unchecked::<LanaApp>();

        let config = TestWaitJobConfig {
            wait_seconds: input.wait_seconds,
            respect_shutdown: input.respect_shutdown,
        };

        let job_id = lana_app::job::JobId::new();
        let _job = app
            .jobs()
            .create_and_spawn::<TestWaitJobConfig>(job_id, config)
            .await?;

        Ok(TestJobSpawnPayload {
            job_id: job_id.into(),
            wait_seconds: input.wait_seconds,
            respect_shutdown: input.respect_shutdown,
        })
    }
}

