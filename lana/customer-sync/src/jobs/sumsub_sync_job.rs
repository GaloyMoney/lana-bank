use job::JobCompletion;

pub(crate) fn complete_on_success<E>(
    result: Result<(), E>,
) -> Result<JobCompletion, Box<dyn std::error::Error>>
where
    E: std::error::Error + Send + Sync + 'static,
{
    result
        .map(|()| JobCompletion::Complete)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

#[cfg(test)]
mod tests {
    use super::complete_on_success;
    use job::JobCompletion;
    use thiserror::Error;

    #[derive(Debug, Error)]
    #[error("sumsub sync failed")]
    struct TestError;

    #[test]
    fn returns_complete_on_success() {
        let result = complete_on_success::<TestError>(Ok(()));

        assert!(matches!(result, Ok(JobCompletion::Complete)));
    }

    #[test]
    fn propagates_errors_for_retry() {
        let result = complete_on_success(Err(TestError));

        let err = result.err().expect("expected error");
        assert_eq!(err.to_string(), "sumsub sync failed");
    }
}
