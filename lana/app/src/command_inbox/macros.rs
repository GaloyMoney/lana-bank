/// Macro to define async commands with minimal boilerplate.
///
/// This macro generates:
/// - Enum variants for CommandInboxPayload
/// - Match arms for the handler
/// - Internal handler methods
/// - Public async methods on CommandInbox
/// - Poll methods for each command
///
/// # Example
///
/// ```ignore
/// define_async_commands! {
///     handlers: {
///         customers: Customers,
///         deposits: Deposits,
///     },
///     commands: [
///         {
///             name: CreateCustomer,
///             id_field: customer_id,
///             id_type: CustomerId,
///             result_type: core_customer::Customer,
///             handler: customers,
///             auth_check: subject_can_create_customer,
///             execute_fn: create_with_id,
///             find_fn: find_by_id_internal,
///             not_found_error: CustomerNotFoundAfterProcessing,
///             fields: {
///                 email: String,
///                 telegram_id: String,
///                 customer_type: CustomerType,
///             },
///         },
///     ]
/// }
/// ```
#[macro_export]
macro_rules! define_async_commands {
    (
        handlers: {
            $($handler_name:ident : $handler_type:ty),* $(,)?
        },
        commands: [
            $(
                {
                    name: $cmd_name:ident,
                    id_field: $id_field:ident,
                    id_type: $id_type:ty,
                    result_type: $result_type:ty,
                    handler: $handler:ident,
                    auth_check: $auth_check:ident,
                    execute_fn: $execute_fn:ident,
                    find_fn: $find_fn:ident,
                    not_found_error: $not_found_error:ident,
                    fields: {
                        $($field_name:ident : $field_type:ty),* $(,)?
                    } $(,)?
                }
            ),* $(,)?
        ]
    ) => {
        // =========================================
        // Generate the CommandInboxPayload enum
        // =========================================
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        pub enum CommandInboxPayload {
            $(
                $cmd_name {
                    $id_field: $id_type,
                    $($field_name: $field_type),*
                },
            )*
        }

        // =========================================
        // Generate the CommandInboxHandler struct
        // =========================================
        struct CommandInboxHandler {
            $($handler_name: $handler_type),*
        }

        impl Clone for CommandInboxHandler {
            fn clone(&self) -> Self {
                Self {
                    $($handler_name: self.$handler_name.clone()),*
                }
            }
        }

        impl obix::inbox::InboxHandler for CommandInboxHandler {
            async fn handle(
                &self,
                event: &obix::inbox::InboxEvent,
            ) -> Result<obix::inbox::InboxResult, Box<dyn std::error::Error + Send + Sync>> {
                let payload: CommandInboxPayload = event.payload()?;
                match payload {
                    $(
                        CommandInboxPayload::$cmd_name { $id_field, $($field_name),* } => {
                            pastey::paste! {
                                self.[<$cmd_name:snake _internal>]($id_field, $($field_name),*).await?;
                            }
                        }
                    )*
                }
                Ok(obix::inbox::InboxResult::Complete)
            }
        }

        impl CommandInboxHandler {
            fn new($($handler_name: &$handler_type),*) -> Self {
                Self {
                    $($handler_name: $handler_name.clone()),*
                }
            }

            // Generate internal methods for each command
            $(
                pastey::paste! {
                    #[tracing_macros::record_error_severity]
                    #[tracing::instrument(skip(self))]
                    async fn [<$cmd_name:snake _internal>](
                        &self,
                        $id_field: $id_type,
                        $($field_name: $field_type),*
                    ) -> Result<$result_type, CommandInboxError> {
                        let result = self.$handler.$execute_fn($id_field, $($field_name),*).await?;
                        Ok(result)
                    }
                }
            )*
        }

        // =========================================
        // Generate the CommandInbox struct
        // =========================================
        pub struct CommandInbox {
            inbox: obix::inbox::Inbox,
            $($handler_name: $handler_type),*
        }

        impl Clone for CommandInbox {
            fn clone(&self) -> Self {
                Self {
                    inbox: self.inbox.clone(),
                    $($handler_name: self.$handler_name.clone()),*
                }
            }
        }

        impl CommandInbox {
            pub async fn init(
                pool: &sqlx::PgPool,
                jobs: &mut $crate::job::Jobs,
                $($handler_name: &$handler_type),*
            ) -> Result<Self, CommandInboxError> {
                let handler = CommandInboxHandler::new($($handler_name),*);
                let inbox_config = obix::inbox::InboxConfig::new(COMMAND_INBOX_JOB);
                let inbox = obix::inbox::Inbox::new(pool, jobs, inbox_config, handler);

                Ok(Self {
                    inbox,
                    $($handler_name: $handler_name.clone()),*
                })
            }

            // Generate async public methods for each command
            $(
                pastey::paste! {
                    #[tracing_macros::record_error_severity]
                    #[tracing::instrument(skip(self))]
                    pub async fn [<$cmd_name:snake _async>](
                        &self,
                        sub: &$crate::primitives::Subject,
                        $($field_name: $field_type),*
                    ) -> Result<$result_type, CommandInboxError> {
                        // Auth check
                        self.$handler.$auth_check(sub, false).await?;

                        // Generate ID upfront for polling
                        let $id_field = <$id_type>::new();

                        // Idempotency key
                        let idempotency_key = format!(
                            "{}:{}",
                            stringify!([<$cmd_name:snake>]),
                            $id_field
                        );

                        // Build payload
                        let payload = CommandInboxPayload::$cmd_name {
                            $id_field,
                            $($field_name),*
                        };

                        // Persist and process
                        let result = self.inbox.persist_and_process(&idempotency_key, payload).await?;

                        match result {
                            es_entity::Idempotent::Executed(_) => {}
                            es_entity::Idempotent::AlreadyApplied => {
                                return Err(CommandInboxError::DuplicateIdempotencyKey);
                            }
                        };

                        // Poll for result
                        let result = self.[<poll_for_ $cmd_name:snake>](
                            $id_field,
                            std::time::Duration::from_millis(100),
                            50
                        ).await?;

                        Ok(result)
                    }

                    async fn [<poll_for_ $cmd_name:snake>](
                        &self,
                        $id_field: $id_type,
                        interval: std::time::Duration,
                        max_attempts: u32,
                    ) -> Result<$result_type, CommandInboxError> {
                        for _ in 0..max_attempts {
                            match self.$handler.$find_fn($id_field).await {
                                Ok(result) => return Ok(result),
                                Err(_) => {
                                    tokio::time::sleep(interval).await;
                                }
                            }
                        }
                        Err(CommandInboxError::$not_found_error)
                    }
                }
            )*
        }
    };
}
