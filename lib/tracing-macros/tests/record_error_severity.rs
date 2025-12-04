use futures::stream::{self, BoxStream, StreamExt};
use std::sync::{Arc, Mutex};
use tracing::subscriber::with_default;
use tracing_macros::record_error_severity;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use tracing_utils::ErrorSeverity;

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error("Critical error")]
    Critical,
    #[error("Warning error")]
    Warning,
}

impl ErrorSeverity for TestError {
    fn severity(&self) -> tracing::Level {
        match self {
            TestError::Critical => tracing::Level::ERROR,
            TestError::Warning => tracing::Level::WARN,
        }
    }
}

type EventLog = Arc<Mutex<Vec<(tracing::Level, String)>>>;

#[derive(Default)]
struct TestCollector {
    events: EventLog,
}

impl TestCollector {
    fn new() -> (Self, EventLog) {
        let events = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                events: events.clone(),
            },
            events,
        )
    }
}

impl<S> tracing_subscriber::Layer<S> for TestCollector
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        use tracing::field::Visit;

        struct Visitor {
            error_msg: Option<String>,
        }

        impl Visit for Visitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "error" {
                    self.error_msg = Some(format!("{:?}", value));
                }
            }
        }

        let mut visitor = Visitor { error_msg: None };
        event.record(&mut visitor);

        if let Some(msg) = visitor.error_msg {
            let mut events = self.events.lock().unwrap();
            events.push((*event.metadata().level(), msg));
        }
    }
}

#[test]
fn test_record_error_severity_macro() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[record_error_severity]
    fn function_that_errors() -> Result<(), TestError> {
        Err(TestError::Critical)
    }

    #[record_error_severity]
    fn function_that_warns() -> Result<(), TestError> {
        Err(TestError::Warning)
    }

    #[record_error_severity]
    fn function_that_succeeds() -> Result<String, TestError> {
        Ok("success".to_string())
    }

    with_default(subscriber, || {
        let span = tracing::span!(
            tracing::Level::INFO,
            "test_span",
            error.level = tracing::field::Empty
        );
        let _enter = span.enter();

        let _ = function_that_errors();
        let _ = function_that_warns();
        let result = function_that_succeeds();
        assert!(result.is_ok());
    });

    let recorded_events = events.lock().unwrap();
    assert_eq!(recorded_events.len(), 2);
    assert_eq!(recorded_events[0].0, tracing::Level::ERROR);
    assert!(recorded_events[0].1.contains("Critical error"));
    assert_eq!(recorded_events[1].0, tracing::Level::WARN);
    assert!(recorded_events[1].1.contains("Warning error"));
}

#[tokio::test]
async fn test_record_error_severity_async() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[record_error_severity]
    async fn async_function_that_errors() -> Result<(), TestError> {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Err(TestError::Critical)
    }

    #[record_error_severity]
    async fn async_function_that_succeeds() -> Result<String, TestError> {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok("async success".to_string())
    }

    let _guard = tracing::subscriber::set_default(subscriber);
    let span = tracing::span!(
        tracing::Level::INFO,
        "async_test_span",
        error.level = tracing::field::Empty
    );
    let _enter = span.enter();

    let _ = async_function_that_errors().await;
    let result = async_function_that_succeeds().await;
    assert!(result.is_ok());

    let recorded_events = events.lock().unwrap();
    assert_eq!(recorded_events.len(), 1);
    assert_eq!(recorded_events[0].0, tracing::Level::ERROR);
    assert!(recorded_events[0].1.contains("Critical error"));
}

#[tokio::test]
async fn test_record_error_severity_with_boxstream() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[derive(Debug)]
    struct TestEvent {
        #[allow(dead_code)]
        id: u64,
    }

    #[record_error_severity]
    async fn function_returning_boxstream() -> Result<BoxStream<'static, Arc<TestEvent>>, TestError>
    {
        // Simulate some async work
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Return a stream
        let stream = stream::iter(vec![
            Arc::new(TestEvent { id: 1 }),
            Arc::new(TestEvent { id: 2 }),
            Arc::new(TestEvent { id: 3 }),
        ]);
        Ok(Box::pin(stream) as BoxStream<'static, Arc<TestEvent>>)
    }

    #[record_error_severity]
    async fn function_returning_boxstream_error(
    ) -> Result<BoxStream<'static, Arc<TestEvent>>, TestError> {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Err(TestError::Critical)
    }

    // Test with lifetime parameter - using a struct to hold the data
    struct StreamHolder {
        events: Vec<Arc<TestEvent>>,
    }

    impl StreamHolder {
        #[record_error_severity]
        async fn function_returning_boxstream_with_lifetime(
            &self,
        ) -> Result<BoxStream<'_, Arc<TestEvent>>, TestError> {
            // Simulate some async work
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;

            // Return a stream that borrows from self
            let stream = stream::iter(self.events.clone());
            Ok(Box::pin(stream))
        }

        #[record_error_severity]
        async fn function_returning_boxstream_with_lifetime_error(
            &self,
        ) -> Result<BoxStream<'_, Arc<TestEvent>>, TestError> {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            Err(TestError::Warning)
        }
    }

    let _guard = tracing::subscriber::set_default(subscriber);

    // Test successful case - should not emit any error events
    let result = function_returning_boxstream().await;
    assert!(result.is_ok());

    // Consume the stream to verify it works
    if let Ok(stream) = result {
        let items: Vec<_> = stream.collect().await;
        assert_eq!(items.len(), 3);
    }

    // Test error case - should emit error event
    let error_result = function_returning_boxstream_error().await;
    assert!(error_result.is_err());

    // Test with lifetime parameter '_ - successful case
    let holder = StreamHolder {
        events: vec![Arc::new(TestEvent { id: 1 }), Arc::new(TestEvent { id: 2 })],
    };

    let lifetime_result = holder.function_returning_boxstream_with_lifetime().await;
    assert!(lifetime_result.is_ok());

    // Consume the stream to verify it works
    if let Ok(stream) = lifetime_result {
        let items: Vec<_> = stream.collect().await;
        assert_eq!(items.len(), 2);
    }

    // Test with lifetime parameter '_ - error case
    let lifetime_error_result = holder
        .function_returning_boxstream_with_lifetime_error()
        .await;
    assert!(lifetime_error_result.is_err());

    let recorded_events = events.lock().unwrap();
    assert_eq!(recorded_events.len(), 2);
    assert_eq!(recorded_events[0].0, tracing::Level::ERROR);
    assert!(recorded_events[0].1.contains("Critical error"));
    assert_eq!(recorded_events[1].0, tracing::Level::WARN);
    assert!(recorded_events[1].1.contains("Warning error"));
}
