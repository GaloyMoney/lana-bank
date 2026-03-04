use std::sync::{Arc, Mutex};
use tracing::subscriber::with_default;
use tracing_macros::observe_error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use tracing_utils::ErrorSeverity;

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error("Critical error")]
    Critical,
    #[error("Warning error")]
    Warning,
    #[error("Info error")]
    Info,
}

impl ErrorSeverity for TestError {
    fn severity(&self) -> tracing::Level {
        match self {
            TestError::Critical => tracing::Level::ERROR,
            TestError::Warning => tracing::Level::WARN,
            TestError::Info => tracing::Level::INFO,
        }
    }

    fn variant_name(&self) -> &'static str {
        match self {
            TestError::Critical => "Critical",
            TestError::Warning => "Warning",
            TestError::Info => "Info",
        }
    }
}

#[derive(Clone)]
struct CapturedEvent {
    level: tracing::Level,
    error_msg: Option<String>,
    error_layer: Option<String>,
    error_boundary: Option<bool>,
    error_use_case: Option<String>,
    error_aggregate: Option<bool>,
}

type EventLog = Arc<Mutex<Vec<CapturedEvent>>>;

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
            error_layer: Option<String>,
            error_boundary: Option<bool>,
            error_use_case: Option<String>,
            error_aggregate: Option<bool>,
        }

        impl Visit for Visitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                match field.name() {
                    "error" => self.error_msg = Some(format!("{:?}", value)),
                    "error.layer" => self.error_layer = Some(format!("{:?}", value)),
                    "error.use_case" => self.error_use_case = Some(format!("{:?}", value)),
                    _ => {}
                }
            }

            fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
                match field.name() {
                    "error.boundary" => self.error_boundary = Some(value),
                    "error.aggregate" => self.error_aggregate = Some(value),
                    _ => {}
                }
            }
        }

        let mut visitor = Visitor {
            error_msg: None,
            error_layer: None,
            error_boundary: None,
            error_use_case: None,
            error_aggregate: None,
        };
        event.record(&mut visitor);

        if visitor.error_msg.is_some() || visitor.error_aggregate.is_some() {
            let mut events = self.events.lock().unwrap();
            events.push(CapturedEvent {
                level: *event.metadata().level(),
                error_msg: visitor.error_msg,
                error_layer: visitor.error_layer,
                error_boundary: visitor.error_boundary,
                error_use_case: visitor.error_use_case,
                error_aggregate: visitor.error_aggregate,
            });
        }
    }
}

// -- Inner mode tests --

#[test]
fn inner_mode_caps_error_severity_to_warn() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[observe_error]
    fn inner_critical() -> Result<(), TestError> {
        Err(TestError::Critical)
    }

    with_default(subscriber, || {
        let _ = inner_critical();
    });

    let recorded = events.lock().unwrap();
    // First event: the capped error event (WARN, not ERROR)
    let primary = &recorded[0];
    assert_eq!(
        primary.level,
        tracing::Level::WARN,
        "ERROR should be capped to WARN in inner mode"
    );
    assert!(primary
        .error_msg
        .as_ref()
        .unwrap()
        .contains("Critical error"));
    assert_eq!(primary.error_layer.as_deref(), Some("\"inner\""));
}

#[test]
fn inner_mode_preserves_warn_severity() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[observe_error]
    fn inner_warn() -> Result<(), TestError> {
        Err(TestError::Warning)
    }

    with_default(subscriber, || {
        let _ = inner_warn();
    });

    let recorded = events.lock().unwrap();
    let primary = &recorded[0];
    assert_eq!(primary.level, tracing::Level::WARN);
    assert!(primary
        .error_msg
        .as_ref()
        .unwrap()
        .contains("Warning error"));
    assert_eq!(primary.error_layer.as_deref(), Some("\"inner\""));
}

#[test]
fn inner_mode_preserves_info_severity() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[observe_error]
    fn inner_info() -> Result<(), TestError> {
        Err(TestError::Info)
    }

    with_default(subscriber, || {
        let _ = inner_info();
    });

    let recorded = events.lock().unwrap();
    let primary = &recorded[0];
    assert_eq!(primary.level, tracing::Level::INFO);
    assert_eq!(primary.error_layer.as_deref(), Some("\"inner\""));
}

// -- Boundary mode tests --

#[test]
fn boundary_mode_preserves_error_severity() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[observe_error(allow_single_error_alert)]
    fn boundary_critical() -> Result<(), TestError> {
        Err(TestError::Critical)
    }

    with_default(subscriber, || {
        let _ = boundary_critical();
    });

    let recorded = events.lock().unwrap();
    let primary = &recorded[0];
    assert_eq!(
        primary.level,
        tracing::Level::ERROR,
        "boundary mode should preserve ERROR severity"
    );
    assert!(primary
        .error_msg
        .as_ref()
        .unwrap()
        .contains("Critical error"));
    assert_eq!(primary.error_boundary, Some(true));
    assert!(primary.error_use_case.is_some());
}

#[test]
fn boundary_mode_preserves_warn_severity() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[observe_error(allow_single_error_alert)]
    fn boundary_warn() -> Result<(), TestError> {
        Err(TestError::Warning)
    }

    with_default(subscriber, || {
        let _ = boundary_warn();
    });

    let recorded = events.lock().unwrap();
    let primary = &recorded[0];
    assert_eq!(primary.level, tracing::Level::WARN);
    assert_eq!(primary.error_boundary, Some(true));
}

// -- General behavior tests --

#[test]
fn non_result_function_passes_through() {
    #[observe_error]
    fn returns_string() -> String {
        "hello".to_string()
    }

    assert_eq!(returns_string(), "hello");
}

#[test]
fn success_result_emits_no_events() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[observe_error]
    fn succeeds() -> Result<String, TestError> {
        Ok("ok".to_string())
    }

    with_default(subscriber, || {
        let result = succeeds();
        assert!(result.is_ok());
    });

    let recorded = events.lock().unwrap();
    assert!(recorded.is_empty(), "success should emit no error events");
}

#[tokio::test]
async fn async_inner_mode_caps_error() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[observe_error]
    async fn async_inner() -> Result<(), TestError> {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        Err(TestError::Critical)
    }

    let _guard = tracing::subscriber::set_default(subscriber);
    let _ = async_inner().await;

    let recorded = events.lock().unwrap();
    let primary = &recorded[0];
    assert_eq!(
        primary.level,
        tracing::Level::WARN,
        "async inner should cap ERROR to WARN"
    );
    assert_eq!(primary.error_layer.as_deref(), Some("\"inner\""));
}

#[tokio::test]
async fn async_boundary_mode_preserves_error() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    #[observe_error(allow_single_error_alert)]
    async fn async_boundary() -> Result<(), TestError> {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        Err(TestError::Critical)
    }

    let _guard = tracing::subscriber::set_default(subscriber);
    let _ = async_boundary().await;

    let recorded = events.lock().unwrap();
    let primary = &recorded[0];
    assert_eq!(primary.level, tracing::Level::ERROR);
    assert_eq!(primary.error_boundary, Some(true));
}

#[tokio::test]
async fn question_mark_early_return_works() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    async fn always_fails() -> Result<String, TestError> {
        Err(TestError::Warning)
    }

    #[observe_error]
    async fn with_question_mark() -> Result<String, TestError> {
        let _val = always_fails().await?;
        Ok("never reached".to_string())
    }

    let _guard = tracing::subscriber::set_default(subscriber);
    let _ = with_question_mark().await;

    let recorded = events.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].level, tracing::Level::WARN);
}

// -- Aggregate threshold test --

#[test]
fn aggregate_threshold_triggers_error_after_sustained_failures() {
    let (collector, events) = TestCollector::new();
    let subscriber = Registry::default().with(collector);

    // Use a unique function name to avoid key collision with other tests
    #[observe_error]
    fn aggregate_test_fn() -> Result<(), TestError> {
        Err(TestError::Warning)
    }

    with_default(subscriber, || {
        // Trigger 10 errors to hit the default threshold (10 in 600s)
        for _ in 0..10 {
            let _ = aggregate_test_fn();
        }
    });

    let recorded = events.lock().unwrap();
    // Each call produces a primary WARN event.
    // The 10th call should also produce an aggregate ERROR event.
    let aggregate_events: Vec<_> = recorded
        .iter()
        .filter(|e| e.error_aggregate == Some(true))
        .collect();

    assert!(
        !aggregate_events.is_empty(),
        "should have at least one aggregate ERROR event after 10 errors"
    );
    assert_eq!(aggregate_events[0].level, tracing::Level::ERROR);
}

// -- aggregate_overrides! test --

#[test]
fn aggregate_overrides_macro_works() {
    tracing_macros::aggregate_overrides!(test_overrides {
        Critical => (3, 60),
        Warning => disabled,
    });

    assert_eq!(test_overrides("Critical"), (3, 60));
    assert_eq!(test_overrides("Warning"), (0, 0));
    assert_eq!(test_overrides("Unknown"), (10, 600));
}
