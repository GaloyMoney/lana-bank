use async_graphql::{Context, Error, MergedObject, Object, Subscription, types::connection::*};

use admin_graphql_access::{AccessMutation, AccessQuery};
use admin_graphql_accounting::{AccountingMutation, AccountingQuery};
use admin_graphql_credit::{CreditMutation, CreditQuery};
use admin_graphql_custody::{CustodyMutation, CustodyQuery};
use admin_graphql_customer::{CustomerMutation, CustomerQuery};
use admin_graphql_deposit::{DepositMutation, DepositQuery};
use admin_graphql_governance::{GovernanceMutation, GovernanceQuery};

use futures::StreamExt;
use futures::stream::Stream;
use obix::out::OutboxEventMarker;

use lana_app::accounting::CoreAccountingEvent;
use lana_app::app::LanaApp;
use lana_app::credit::CoreCreditEvent;
use lana_app::price::CorePriceEvent;
use lana_app::report::CoreReportEvent;

use crate::primitives::*;

use super::{
    access::User, accounting::*, audit::*, credit_facility::*, customer::*, dashboard::*,
    deposit::*, domain_config::*, loader::CHART_REF, loader::*, me::*, price::*, prospect::*,
    public_id::*, reports::*, withdrawal::*,
};

#[derive(MergedObject, Default)]
pub struct Query(
    pub AccessQuery,
    pub AccountingQuery,
    pub CreditQuery,
    pub CustomerQuery,
    pub CustodyQuery,
    pub DepositQuery,
    pub GovernanceQuery,
    pub BaseQuery,
);

#[derive(Default)]
pub struct BaseQuery;

#[Object]
impl BaseQuery {
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<MeUser> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let user = Arc::new(app.access().users().find_for_subject(sub).await?);
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        loader.feed_one(user.id, User::from(user.clone())).await;
        Ok(MeUser::from(user))
    }

    async fn dashboard(&self, ctx: &Context<'_>) -> async_graphql::Result<Dashboard> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let dashboard = app.dashboard().load(sub).await?;
        Ok(Dashboard::from(dashboard))
    }

    async fn realtime_price(&self, ctx: &Context<'_>) -> async_graphql::Result<RealtimePrice> {
        let app = ctx.data_unchecked::<LanaApp>();
        let usd_cents_per_btc = app.price().usd_cents_per_btc().await;
        Ok(usd_cents_per_btc.into())
    }

    async fn audit(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        subject: Option<AuditSubjectId>,
        authorized: Option<bool>,
        object: Option<String>,
        action: Option<String>,
    ) -> async_graphql::Result<Connection<AuditCursor, AuditEntry>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let subject_filter: Option<String> = subject.map(String::from);
        let authorized_filter = authorized;
        let object_filter = object;
        let action_filter = action;
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .list_audit(
                        sub,
                        es_entity::PaginatedQueryArgs {
                            first,
                            after: after.map(lana_app::audit::AuditCursor::from),
                        },
                        subject_filter.clone(),
                        authorized_filter,
                        object_filter.clone(),
                        action_filter.clone(),
                    )
                    .await?;

                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entry| {
                        let cursor = AuditCursor::from(&entry);
                        Edge::new(cursor, AuditEntry::from(entry))
                    }));

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn audit_subjects(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<AuditSubjectId>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app
            .list_audit_subjects(sub)
            .await?
            .into_iter()
            .map(AuditSubjectId::from)
            .collect())
    }

    async fn domain_configs(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DomainConfigsByKeyCursor, DomainConfig, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            DomainConfigsByKeyCursor,
            DomainConfig,
            ctx,
            after,
            first,
            |query| app.exposed_domain_configs().list(sub, query)
        )
    }

    async fn public_id_target(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<PublicIdTarget>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let Some(public_id) = app.public_ids().find_by_id(id).await? else {
            return Ok(None);
        };

        let res = match public_id.target_type.as_str() {
            "customer" => {
                let customer: Option<Customer> = app
                    .customers()
                    .find_all::<Customer>(&[public_id.target_id.into()])
                    .await?
                    .into_values()
                    .next();
                customer.map(PublicIdTarget::Customer)
            }
            "deposit_account" => {
                let account: Option<DepositAccount> = app
                    .deposits()
                    .find_all_deposit_accounts::<DepositAccount>(&[public_id.target_id.into()])
                    .await?
                    .into_values()
                    .next();
                account.map(PublicIdTarget::DepositAccount)
            }
            "deposit" => {
                let deposit: Option<Deposit> = app
                    .deposits()
                    .find_all_deposits::<Deposit>(&[public_id.target_id.into()])
                    .await?
                    .into_values()
                    .next();
                deposit.map(PublicIdTarget::Deposit)
            }
            "withdrawal" => {
                let withdrawal: Option<Withdrawal> = app
                    .deposits()
                    .find_all_withdrawals::<Withdrawal>(&[public_id.target_id.into()])
                    .await?
                    .into_values()
                    .next();
                withdrawal.map(PublicIdTarget::Withdrawal)
            }
            "credit_facility" => {
                let facility: Option<CreditFacility> = app
                    .credit()
                    .facilities()
                    .find_all::<CreditFacility>(&[public_id.target_id.into()])
                    .await?
                    .into_values()
                    .next();
                facility.map(PublicIdTarget::CreditFacility)
            }
            "disbursal" => {
                let disbursal: Option<CreditFacilityDisbursal> = app
                    .credit()
                    .disbursals()
                    .find_all::<CreditFacilityDisbursal>(&[public_id.target_id.into()])
                    .await?
                    .into_values()
                    .next();
                disbursal.map(PublicIdTarget::CreditFacilityDisbursal)
            }
            "prospect" => {
                let prospect: Option<Prospect> = app
                    .customers()
                    .find_all_prospects::<Prospect>(&[public_id.target_id.into()])
                    .await?
                    .into_values()
                    .next();
                prospect.map(PublicIdTarget::Prospect)
            }
            _ => None,
        };
        Ok(res)
    }

    async fn report_runs(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<ReportRunsByCreatedAtCursor, ReportRun, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            ReportRunsByCreatedAtCursor,
            ReportRun,
            ctx,
            after,
            first,
            |query| app.reports().list_report_runs(sub, query)
        )
    }

    async fn report_run(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<ReportRun>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(ReportRun, ctx, app.reports().find_report_run_by_id(sub, id))
    }
}

#[derive(MergedObject, Default)]
pub struct Mutation(
    pub AccessMutation,
    pub AccountingMutation,
    pub CreditMutation,
    pub CustomerMutation,
    pub CustodyMutation,
    pub DepositMutation,
    pub GovernanceMutation,
    pub BaseMutation,
);

#[derive(Default)]
pub struct BaseMutation;

#[Object]
impl BaseMutation {
    async fn domain_config_update(
        &self,
        ctx: &Context<'_>,
        input: DomainConfigUpdateInput,
    ) -> async_graphql::Result<DomainConfigUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            DomainConfigUpdatePayload,
            DomainConfig,
            ctx,
            app.exposed_domain_configs().update_from_json(
                sub,
                input.domain_config_id,
                input.value.into_inner(),
            )
        )
    }

    async fn trigger_report_run(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<ReportRunCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let _job_id = app.reports().trigger_report_run_job(sub).await?;
        Ok(ReportRunCreatePayload { run_id: None })
    }

    async fn report_file_generate_download_link(
        &self,
        ctx: &Context<'_>,
        input: ReportFileGenerateDownloadLinkInput,
    ) -> async_graphql::Result<ReportFileGenerateDownloadLinkPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let url = app
            .reports()
            .generate_report_file_download_link(sub, input.report_id, input.extension)
            .await?;
        Ok(ReportFileGenerateDownloadLinkPayload { url })
    }
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn pending_credit_facility_collateralization_updated(
        &self,
        ctx: &Context<'_>,
        pending_credit_facility_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = PendingCreditFacilityCollateralizationPayload>>
    {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let pending_credit_facility_id = PendingCreditFacilityId::from(pending_credit_facility_id);

        app.credit()
            .pending_credit_facilities()
            .find_by_id(sub, pending_credit_facility_id)
            .await?;

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |message| async move {
            let payload = message.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::PendingCreditFacilityCollateralizationChanged { entity }
                    if entity.id == pending_credit_facility_id =>
                {
                    let collateralization = &entity.collateralization;
                    Some(PendingCreditFacilityCollateralizationPayload {
                        pending_credit_facility_id,
                        update: PendingCreditFacilityCollateralizationUpdated {
                            state: collateralization.state,
                            collateral: collateralization.collateral.expect("collateral must be set for PendingCreditFacilityCollateralizationChanged"),
                            price: collateralization.price_at_state_change.expect("price must be set for PendingCreditFacilityCollateralizationChanged").into_inner(),
                            recorded_at: message.recorded_at.into(),
                            effective: message.recorded_at.date_naive().into(),
                        },
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn pending_credit_facility_completed(
        &self,
        ctx: &Context<'_>,
        pending_credit_facility_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = PendingCreditFacilityCompletedPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let pending_credit_facility_id = PendingCreditFacilityId::from(pending_credit_facility_id);

        app.credit()
            .pending_credit_facilities()
            .find_by_id(sub, pending_credit_facility_id)
            .await?;

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |event| async move {
            let payload = event.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::PendingCreditFacilityCompleted { entity }
                    if entity.id == pending_credit_facility_id =>
                {
                    Some(PendingCreditFacilityCompletedPayload {
                        pending_credit_facility_id,
                        update: PendingCreditFacilityCompleted {
                            status: entity.status,
                            recorded_at: entity.completed_at?.into(),
                        },
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn credit_facility_proposal_concluded(
        &self,
        ctx: &Context<'_>,
        credit_facility_proposal_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = CreditFacilityProposalConcludedPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let credit_facility_proposal_id =
            CreditFacilityProposalId::from(credit_facility_proposal_id);

        app.credit()
            .proposals()
            .find_by_id(sub, credit_facility_proposal_id)
            .await?
            .ok_or_else(|| Error::new("Credit facility proposal not found"))?;

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |event| async move {
            let payload = event.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::FacilityProposalConcluded { entity }
                    if entity.id == credit_facility_proposal_id =>
                {
                    Some(CreditFacilityProposalConcludedPayload {
                        credit_facility_proposal_id,
                        status: entity.status,
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn credit_facility_collateralization_updated(
        &self,
        ctx: &Context<'_>,
        credit_facility_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = CreditFacilityCollateralizationPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let credit_facility_id = CreditFacilityId::from(credit_facility_id);

        app.credit()
            .facilities()
            .find_by_id(sub, credit_facility_id)
            .await?;

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |message| async move {
            let payload = message.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::FacilityCollateralizationChanged { entity }
                    if entity.id == credit_facility_id =>
                {
                    let collateralization = &entity.collateralization;
                    Some(CreditFacilityCollateralizationPayload {
                        credit_facility_id,
                        update: CreditFacilityCollateralizationUpdated {
                            state: collateralization.state,
                            collateral: collateralization.collateral,
                            outstanding_interest: collateralization.outstanding.interest,
                            outstanding_disbursal: collateralization.outstanding.disbursed,
                            recorded_at: message.recorded_at.into(),
                            effective: message.recorded_at.date_naive().into(),
                            price: collateralization.price_at_state_change.into_inner(),
                        },
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn ledger_account_csv_export_uploaded(
        &self,
        ctx: &Context<'_>,
        ledger_account_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = LedgerAccountCsvExportUploadedPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let ledger_account_id = LedgerAccountId::from(ledger_account_id);

        app.accounting()
            .find_ledger_account_by_id(sub, CHART_REF.0, ledger_account_id)
            .await?
            .ok_or_else(|| Error::new("Ledger account not found"))?;

        let stream = app.outbox().listen_ephemeral();
        let updates = stream.filter_map(move |event| async move {
            let event: &CoreAccountingEvent = event.payload.as_event()?;
            match event {
                CoreAccountingEvent::LedgerAccountCsvExportUploaded {
                    id,
                    ledger_account_id: event_ledger_account_id,
                } if *event_ledger_account_id == ledger_account_id => {
                    Some(LedgerAccountCsvExportUploadedPayload {
                        document_id: UUID::from(*id),
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn realtime_price_updated(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<impl Stream<Item = RealtimePrice>> {
        let app = ctx.data_unchecked::<LanaApp>();

        let stream = app.outbox().listen_ephemeral();
        let updates = stream.filter_map(move |event| async move {
            let event: &CorePriceEvent = event.payload.as_event()?;
            match event {
                CorePriceEvent::PriceUpdated { price, .. } => Some(RealtimePrice::from(*price)),
            }
        });

        Ok(updates)
    }

    async fn report_run_updated(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<impl Stream<Item = ReportRunUpdatedPayload>> {
        let app = ctx.data_unchecked::<LanaApp>();

        let stream = app.outbox().listen_ephemeral();
        let updates = stream.filter_map(move |event| async move {
            let event: &CoreReportEvent = event.payload.as_event()?;
            match event {
                CoreReportEvent::ReportRunCreated { entity }
                | CoreReportEvent::ReportRunStateUpdated { entity } => {
                    Some(ReportRunUpdatedPayload {
                        report_run_id: UUID::from(entity.id),
                    })
                }
            }
        });

        Ok(updates)
    }
}
