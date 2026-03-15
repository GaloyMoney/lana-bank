#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]
mod config;
pub mod error;
mod event;
pub mod jobs;
mod primitives;
pub mod provider;

use std::collections::HashMap;

use futures::StreamExt;
use job::Jobs;
use obix::out::{EphemeralOutboxEvent, Outbox, OutboxEventMarker};
use std::sync::Arc;
use tokio::{sync::watch, task::JoinHandle};
use tracing::Span;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::clock::ClockHandle;

pub use config::*;
use error::PriceError;

pub use event::*;
pub use jobs::get_price_from_bfx;
pub use primitives::*;
pub use provider::{
    PriceProvider, PriceProviderConfig, PriceProvidersSortBy, price_provider_cursor,
};

use jobs::fetch_price;
use provider::{NewPriceProvider, PriceProviderRepo};

pub const PRICE_BOOTSTRAP: audit::SystemActor = audit::SystemActor::new("price-bootstrap");

#[derive(Clone)]
pub struct Price {
    receiver: watch::Receiver<Option<PriceOfOneBTC>>,
    _handle: Arc<JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
}

impl Price {
    pub fn new<E>(outbox: &Outbox<E>) -> Self
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        let (tx, rx) = watch::channel(None);
        let handle = Self::spawn_price_listener(tx, outbox.clone());
        Self {
            receiver: rx,
            _handle: Arc::new(handle),
        }
    }

    pub async fn usd_cents_per_btc(&self) -> PriceOfOneBTC {
        let mut rec = self.receiver.clone();
        loop {
            if let Some(res) = *rec.borrow() {
                return res;
            }
            let _ = rec.changed().await;
        }
    }

    fn spawn_price_listener<E>(
        tx: watch::Sender<Option<PriceOfOneBTC>>,
        outbox: Outbox<E>,
    ) -> JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        tokio::spawn(Self::listen_for_price_updates(tx, outbox))
    }

    #[tracing::instrument(name = "core.price.listen_for_updates", skip_all, err)]
    async fn listen_for_price_updates<E>(
        tx: watch::Sender<Option<PriceOfOneBTC>>,
        outbox: Outbox<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        let mut stream = outbox.listen_ephemeral();

        while let Some(message) = stream.next().await {
            Self::process_message(&tx, message.as_ref()).await?;
        }

        tracing::info!("price outbox listener stream ended");
        Ok(())
    }

    #[tracing::instrument(
        name = "core.price.listen_for_updates.process_message",
        parent = None,
        skip(tx, message),
        fields(event_type = tracing::field::Empty, handled = false, price = tracing::field::Empty, timestamp = tracing::field::Empty),
        err
    )]
    async fn process_message<E>(
        tx: &watch::Sender<Option<PriceOfOneBTC>>,
        message: &EphemeralOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        E: OutboxEventMarker<CorePriceEvent> + Send + Sync + 'static,
    {
        if let Some(CorePriceEvent::PriceUpdated {
            price: new_price,
            timestamp,
        }) = message.payload.as_event()
        {
            Span::current().record("handled", true);
            Span::current().record("event_type", "PriceUpdated");
            Span::current().record("price", tracing::field::display(new_price));
            Span::current().record("timestamp", tracing::field::debug(timestamp));
            tx.send(Some(*new_price))?;
        }

        Ok(())
    }
}

pub struct CorePrice<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CorePriceEvent>,
{
    authz: Perms,
    providers: PriceProviderRepo,
    price: Price,
    jobs: Jobs,
    outbox: Outbox<E>,
}

impl<Perms, E> CorePrice<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CorePriceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CorePriceObject>,
    E: OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "core.price.init", skip_all)]
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        jobs: &mut Jobs,
        outbox: &Outbox<E>,
        clock: ClockHandle,
    ) -> Result<Self, PriceError>
    where
        E: Send + Sync + 'static,
    {
        let providers = PriceProviderRepo::new(pool, clock);
        let price = Price::new(outbox);

        let fetch_job_spawner =
            jobs.add_initializer(fetch_price::FetchPriceJobInit::new(&providers, outbox));

        // Auto-bootstrap: if no providers exist, create a default Bitfinex provider
        let mut db = providers.begin_op().await?;
        let existing = providers
            .list_by_id_in_op(&mut db, Default::default(), Default::default())
            .await?;
        if existing.entities.is_empty() {
            let config = PriceProviderConfig::Bitfinex;
            let config_value =
                serde_json::to_value(&config).expect("PriceProviderConfig serializes");
            let id = PriceProviderId::new();
            let new_provider = NewPriceProvider::builder()
                .id(id)
                .name("Bitfinex".to_string())
                .provider(provider::PriceProviderConfigDiscriminants::from(&config).to_string())
                .provider_config(config_value)
                .build()
                .expect("should always build a new price provider");

            authz
                .audit()
                .record_system_entry_in_op(
                    &mut db,
                    PRICE_BOOTSTRAP,
                    CorePriceObject::all_providers(),
                    CorePriceAction::PROVIDER_CREATE,
                )
                .await?;

            providers.create_in_op(&mut db, new_provider).await?;
        }
        db.commit().await?;

        // Spawn a single fetch job (the runner loads the active provider from the repo)
        fetch_job_spawner
            .spawn_unique(
                job::JobId::new(),
                fetch_price::FetchPriceJobConfig::<E> {
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

        Ok(Self {
            authz: authz.clone(),
            providers,
            price,
            jobs: jobs.clone(),
            outbox: outbox.clone(),
        })
    }

    pub fn price(&self) -> &Price {
        &self.price
    }

    #[record_error_severity]
    #[tracing::instrument(name = "core.price.create_provider", skip(self))]
    pub async fn create_provider(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
        config: PriceProviderConfig,
    ) -> Result<provider::PriceProvider, PriceError>
    where
        E: Send + Sync + 'static,
    {
        self.authz
            .enforce_permission(
                sub,
                CorePriceObject::all_providers(),
                CorePriceAction::PROVIDER_CREATE,
            )
            .await?;

        let config_value = serde_json::to_value(&config).expect("PriceProviderConfig serializes");
        let id = PriceProviderId::new();
        let new_provider = NewPriceProvider::builder()
            .id(id)
            .name(name)
            .provider(provider::PriceProviderConfigDiscriminants::from(&config).to_string())
            .provider_config(config_value)
            .build()
            .expect("should always build a new price provider");

        let provider = self.providers.create(new_provider).await?;

        Ok(provider)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "core.price.update_provider_config", skip(self))]
    pub async fn update_provider_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<PriceProviderId> + std::fmt::Debug,
        config: PriceProviderConfig,
    ) -> Result<provider::PriceProvider, PriceError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CorePriceObject::provider(id),
                CorePriceAction::PROVIDER_UPDATE,
            )
            .await?;

        let mut op = self.providers.begin_op().await?;
        let mut provider = self.providers.find_by_id_in_op(&mut op, id).await?;

        if provider.update_config(config).did_execute() {
            self.providers.update_in_op(&mut op, &mut provider).await?;
            op.commit().await?;
        }

        Ok(provider)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "core.price.list_providers", skip(self))]
    pub async fn list_providers(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<price_provider_cursor::PriceProvidersCursor>,
        sort: es_entity::Sort<PriceProvidersSortBy>,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            provider::PriceProvider,
            price_provider_cursor::PriceProvidersCursor,
        >,
        PriceError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CorePriceObject::all_providers(),
                CorePriceAction::PROVIDER_LIST,
            )
            .await?;
        Ok(self
            .providers
            .list_for_filters(Default::default(), sort, query)
            .await?)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "core.price.find_all_providers", skip(self))]
    pub async fn find_all_providers<T: From<provider::PriceProvider>>(
        &self,
        ids: &[PriceProviderId],
    ) -> Result<HashMap<PriceProviderId, T>, PriceError> {
        Ok(self.providers.find_all(ids).await?)
    }
}

impl<Perms, E> Clone for CorePrice<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CorePriceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            providers: self.providers.clone(),
            price: self.price.clone(),
            jobs: self.jobs.clone(),
            outbox: self.outbox.clone(),
        }
    }
}
