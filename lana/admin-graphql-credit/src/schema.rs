use async_graphql::{Context, Error, Object, Subscription, types::connection::*};
use futures::{StreamExt, stream::Stream};

use obix::out::OutboxEventMarker;

use super::*;
use lana_app::{credit::CoreCreditEvent, credit::LiquidationsByIdCursor};

#[derive(Default)]
pub struct CreditQuery;

#[Object]
impl CreditQuery {
    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacility>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacility,
            ctx,
            app.credit().facilities().find_by_id(sub, id)
        )
    }

    async fn credit_facility_proposal(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacilityProposal>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        maybe_fetch_one!(
            CreditFacilityProposal,
            ctx,
            app.credit().proposals().find_by_id(sub, id)
        )
    }

    async fn credit_facility_proposals(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<
            CreditFacilityProposalsByCreatedAtCursor,
            CreditFacilityProposal,
            EmptyFields,
            EmptyFields,
        >,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            CreditFacilityProposalsByCreatedAtCursor,
            CreditFacilityProposal,
            ctx,
            after,
            first,
            |query| app.credit().proposals().list(sub, query)
        )
    }

    async fn pending_credit_facility(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<PendingCreditFacility>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        maybe_fetch_one!(
            PendingCreditFacility,
            ctx,
            app.credit().pending_credit_facilities().find_by_id(sub, id)
        )
    }

    async fn pending_credit_facilities(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<
            PendingCreditFacilitiesByCreatedAtCursor,
            PendingCreditFacility,
            EmptyFields,
            EmptyFields,
        >,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            PendingCreditFacilitiesByCreatedAtCursor,
            PendingCreditFacility,
            ctx,
            after,
            first,
            |query| app.credit().pending_credit_facilities().list(sub, query)
        )
    }

    async fn credit_facility_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<CreditFacility>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacility,
            ctx,
            app.credit().facilities().find_by_public_id(sub, id)
        )
    }

    async fn credit_facilities(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        #[graphql(default_with = "Some(CreditFacilitiesSort::default())")] sort: Option<
            CreditFacilitiesSort,
        >,
        filter: Option<CreditFacilitiesFilter>,
    ) -> async_graphql::Result<
        Connection<CreditFacilitiesCursor, CreditFacility, EmptyFields, EmptyFields>,
    > {
        let filter = DomainCreditFacilitiesFilters {
            status: filter.as_ref().and_then(|f| f.status),
            collateralization_state: filter.as_ref().and_then(|f| f.collateralization_state),
            customer_id: None,
        };

        let sort = sort.unwrap_or_default();
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_combo_cursor!(
            CreditFacilitiesCursor,
            CreditFacility,
            DomainCreditFacilitiesSortBy::from(sort),
            ctx,
            after,
            first,
            |query| app.credit().facilities().list(sub, query, filter, sort)
        )
    }

    async fn disbursal(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacilityDisbursal>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacilityDisbursal,
            ctx,
            app.credit().disbursals().find_by_id(sub, id)
        )
    }

    async fn disbursal_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<CreditFacilityDisbursal>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacilityDisbursal,
            ctx,
            app.credit().disbursals().find_by_public_id(sub, id)
        )
    }

    async fn disbursals(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DisbursalsCursor, CreditFacilityDisbursal, EmptyFields, EmptyFields>,
    > {
        let filter = DisbursalsFilters::default();

        let sort = Sort {
            by: DomainDisbursalsSortBy::CreatedAt,
            direction: ListDirection::Descending,
        };
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_combo_cursor!(
            DisbursalsCursor,
            CreditFacilityDisbursal,
            sort.by,
            ctx,
            after,
            first,
            |query| { app.credit().disbursals().list(sub, query, filter, sort) }
        )
    }

    async fn liquidation(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Liquidation>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            Liquidation,
            ctx,
            app.credit().collaterals().find_liquidation_by_id(sub, id)
        )
    }

    async fn liquidations(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<LiquidationsByIdCursor, Liquidation, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            LiquidationsByIdCursor,
            Liquidation,
            ctx,
            after,
            first,
            |query| app.credit().collaterals().list_liquidations(sub, query)
        )
    }
}

#[derive(Default)]
pub struct CreditMutation;

#[Object]
impl CreditMutation {
    pub async fn credit_facility_proposal_create(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityProposalCreateInput,
    ) -> async_graphql::Result<CreditFacilityProposalCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CreditFacilityProposalCreateInput {
            facility,
            customer_id,
            terms,
            custodian_id,
        } = input;

        let credit_facility_term_values = lana_app::terms::TermValues::builder()
            .annual_rate(terms.annual_rate)
            .accrual_interval(terms.accrual_interval)
            .accrual_cycle_interval(terms.accrual_cycle_interval)
            .one_time_fee_rate(terms.one_time_fee_rate)
            .disbursal_policy(terms.disbursal_policy)
            .duration(terms.duration)
            .interest_due_duration_from_accrual(terms.interest_due_duration_from_accrual)
            .obligation_overdue_duration_from_due(terms.obligation_overdue_duration_from_due)
            .obligation_liquidation_duration_from_due(
                terms.obligation_liquidation_duration_from_due,
            )
            .liquidation_cvl(terms.liquidation_cvl)
            .margin_call_cvl(terms.margin_call_cvl)
            .initial_cvl(terms.initial_cvl)
            .build()?;

        exec_mutation!(
            CreditFacilityProposalCreatePayload,
            CreditFacilityProposal,
            ctx,
            app.create_facility_proposal(
                sub,
                customer_id,
                facility,
                credit_facility_term_values,
                custodian_id
            )
        )
    }

    pub async fn credit_facility_proposal_customer_approval_conclude(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityProposalCustomerApprovalConcludeInput,
    ) -> async_graphql::Result<CreditFacilityProposalCustomerApprovalConcludePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CreditFacilityProposalCustomerApprovalConcludeInput {
            credit_facility_proposal_id,
            approved,
        } = input;

        exec_mutation!(
            CreditFacilityProposalCustomerApprovalConcludePayload,
            CreditFacilityProposal,
            ctx,
            app.credit().proposals().conclude_customer_approval(
                sub,
                credit_facility_proposal_id,
                approved
            )
        )
    }

    pub async fn collateral_update(
        &self,
        ctx: &Context<'_>,
        input: CollateralUpdateInput,
    ) -> async_graphql::Result<CollateralUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CollateralUpdateInput {
            collateral_id,
            collateral,
            effective,
        } = input;
        exec_mutation!(
            CollateralUpdatePayload,
            Collateral,
            ctx,
            app.credit().collaterals().update_collateral_by_id(
                sub,
                collateral_id.into(),
                collateral,
                effective.into()
            )
        )
    }

    pub async fn credit_facility_partial_payment_record(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityPartialPaymentRecordInput,
    ) -> async_graphql::Result<CreditFacilityPartialPaymentRecordPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityPartialPaymentRecordPayload,
            CreditFacility,
            ctx,
            app.record_payment(sub, input.credit_facility_id, input.amount,)
        )
    }

    pub async fn credit_facility_partial_payment_with_date_record(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityPartialPaymentWithDateRecordInput,
    ) -> async_graphql::Result<CreditFacilityPartialPaymentRecordPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityPartialPaymentRecordPayload,
            CreditFacility,
            ctx,
            app.record_payment_with_date(
                sub,
                input.credit_facility_id,
                input.amount,
                input.effective
            )
        )
    }

    pub async fn credit_facility_disbursal_initiate(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityDisbursalInitiateInput,
    ) -> async_graphql::Result<CreditFacilityDisbursalInitiatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityDisbursalInitiatePayload,
            CreditFacilityDisbursal,
            ctx,
            app.credit()
                .initiate_disbursal(sub, input.credit_facility_id.into(), input.amount)
        )
    }

    async fn credit_facility_complete(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityCompleteInput,
    ) -> async_graphql::Result<CreditFacilityCompletePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityCompletePayload,
            CreditFacility,
            ctx,
            app.credit()
                .complete_facility(sub, input.credit_facility_id)
        )
    }

    async fn collateral_record_sent_to_liquidation(
        &self,
        ctx: &Context<'_>,
        input: CollateralRecordSentToLiquidationInput,
    ) -> async_graphql::Result<CollateralRecordSentToLiquidationPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CollateralRecordSentToLiquidationPayload,
            Collateral,
            ctx,
            app.credit()
                .collaterals()
                .record_collateral_update_via_liquidation(
                    sub,
                    input.collateral_id.into(),
                    input.amount
                )
        )
    }

    async fn collateral_record_proceeds_from_liquidation(
        &self,
        ctx: &Context<'_>,
        input: CollateralRecordProceedsFromLiquidationInput,
    ) -> async_graphql::Result<CollateralRecordProceedsFromLiquidationPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CollateralRecordProceedsFromLiquidationPayload,
            Collateral,
            ctx,
            app.credit()
                .collaterals()
                .record_proceeds_received_and_liquidation_completed(
                    sub,
                    input.collateral_id.into(),
                    input.amount
                )
        )
    }
}

#[derive(Default)]
pub struct CreditSubscription;

#[Subscription]
impl CreditSubscription {
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
}
