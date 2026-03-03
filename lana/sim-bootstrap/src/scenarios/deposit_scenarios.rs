use rust_decimal_macros::dec;
use tracing::{event, instrument};

use lana_app::{app::LanaApp, primitives::*};

use crate::helpers;

#[instrument(name = "sim_bootstrap.deposit_scenarios.run", skip(app), err)]
pub async fn run(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    event!(tracing::Level::INFO, "Starting deposit scenarios");

    scenario_s04_deposits(sub, app).await?;
    scenario_s05_withdrawals(sub, app).await?;
    scenario_s06_deposit_reversal(sub, app).await?;
    scenario_s11_frozen_account(sub, app).await?;
    scenario_s12_closed_account(sub, app).await?;
    scenario_s14_multiple_accounts(sub, app).await?;
    scenario_s16_zero_balance(sub, app).await?;
    scenario_s18_deposit_only(sub, app).await?;
    scenario_s19_many_transactions(sub, app).await?;

    event!(tracing::Level::INFO, "Deposit scenarios completed");
    Ok(())
}

/// S04: Deposits + positive balance
/// Two deposits totaling 0.8 BTC worth of USD
#[instrument(name = "sim_bootstrap.deposit_scenarios.s04", skip(app), err)]
async fn scenario_s04_deposits(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (_, deposit_account_id) =
        helpers::create_customer(sub, app, "-nrsf03-s04-deposits").await?;

    let deposit_1_amount = UsdCents::try_from_usd(dec!(50_000))?;
    let deposit_2_amount = UsdCents::try_from_usd(dec!(30_000))?;

    app.deposits()
        .record_deposit(sub, deposit_account_id, deposit_1_amount, None)
        .await?;
    app.deposits()
        .record_deposit(sub, deposit_account_id, deposit_2_amount, None)
        .await?;

    event!(tracing::Level::INFO, "S04: Two deposits recorded");
    Ok(())
}

/// S05: Withdrawals reducing balance
/// Deposit then withdraw, leaving a reduced balance
#[instrument(name = "sim_bootstrap.deposit_scenarios.s05", skip(app), err)]
async fn scenario_s05_withdrawals(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (_, deposit_account_id) =
        helpers::create_customer(sub, app, "-nrsf03-s05-withdrawals").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(100_000))?;
    app.deposits()
        .record_deposit(sub, deposit_account_id, deposit_amount, None)
        .await?;

    let withdrawal_amount = UsdCents::try_from_usd(dec!(20_000))?;
    let withdrawal = app
        .deposits()
        .initiate_withdrawal(sub, deposit_account_id, withdrawal_amount, None)
        .await?;

    helpers::approve_and_confirm_withdrawal(sub, app, withdrawal.id).await?;

    event!(tracing::Level::INFO, "S05: Withdrawal completed");
    Ok(())
}

/// S06: Deposit reversal
/// Deposit 200k, deposit 500k (erroneous), revert the second deposit
/// Also used by S07 and S08 (same customer)
#[instrument(name = "sim_bootstrap.deposit_scenarios.s06", skip(app), err)]
async fn scenario_s06_deposit_reversal(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (_, deposit_account_id) =
        helpers::create_customer(sub, app, "-nrsf03-s06-deposit-reversal").await?;

    let deposit_1_amount = UsdCents::try_from_usd(dec!(200_000))?;
    app.deposits()
        .record_deposit(sub, deposit_account_id, deposit_1_amount, None)
        .await?;

    let deposit_2_amount = UsdCents::try_from_usd(dec!(500_000))?;
    let erroneous_deposit = app
        .deposits()
        .record_deposit(sub, deposit_account_id, deposit_2_amount, None)
        .await?;

    app.deposits()
        .revert_deposit(sub, erroneous_deposit.id)
        .await?;

    event!(
        tracing::Level::INFO,
        "S06: Erroneous deposit reverted, balance restored to 200k"
    );

    // S07: Withdrawal reversal (same customer/account)
    scenario_s07_withdrawal_reversal(sub, app, deposit_account_id).await?;

    // S08: Cancelled withdrawal (same customer/account)
    scenario_s08_cancelled_withdrawal(sub, app, deposit_account_id).await?;

    Ok(())
}

/// S07: Withdrawal reversal
/// Withdraw 50k, confirm, then revert the withdrawal
#[instrument(name = "sim_bootstrap.deposit_scenarios.s07", skip(app), err)]
async fn scenario_s07_withdrawal_reversal(
    sub: &Subject,
    app: &LanaApp,
    deposit_account_id: DepositAccountId,
) -> anyhow::Result<()> {
    let withdrawal_amount = UsdCents::try_from_usd(dec!(50_000))?;
    let withdrawal = app
        .deposits()
        .initiate_withdrawal(sub, deposit_account_id, withdrawal_amount, None)
        .await?;

    let confirmed = helpers::approve_and_confirm_withdrawal(sub, app, withdrawal.id).await?;

    app.deposits().revert_withdrawal(sub, confirmed.id).await?;

    event!(
        tracing::Level::INFO,
        "S07: Withdrawal confirmed then reverted"
    );
    Ok(())
}

/// S08: Cancelled withdrawal
/// Initiate a withdrawal then cancel it before confirmation
#[instrument(name = "sim_bootstrap.deposit_scenarios.s08", skip(app), err)]
async fn scenario_s08_cancelled_withdrawal(
    sub: &Subject,
    app: &LanaApp,
    deposit_account_id: DepositAccountId,
) -> anyhow::Result<()> {
    let withdrawal_amount = UsdCents::try_from_usd(dec!(100_000))?;
    let withdrawal = app
        .deposits()
        .initiate_withdrawal(sub, deposit_account_id, withdrawal_amount, None)
        .await?;

    app.deposits().cancel_withdrawal(sub, withdrawal.id).await?;

    event!(tracing::Level::INFO, "S08: Withdrawal cancelled");
    Ok(())
}

/// S11: Frozen account (freeze + unfreeze)
#[instrument(name = "sim_bootstrap.deposit_scenarios.s11", skip(app), err)]
async fn scenario_s11_frozen_account(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (_, deposit_account_id) =
        helpers::create_customer(sub, app, "-nrsf03-s11-frozen-account").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(50_000))?;
    app.deposits()
        .record_deposit(sub, deposit_account_id, deposit_amount, None)
        .await?;

    app.deposits()
        .freeze_account(sub, deposit_account_id)
        .await?;

    app.deposits()
        .unfreeze_account(sub, deposit_account_id)
        .await?;

    event!(tracing::Level::INFO, "S11: Account frozen and unfrozen");
    Ok(())
}

/// S12: Closed account
/// Deposit, withdraw full balance, then close the account
#[instrument(name = "sim_bootstrap.deposit_scenarios.s12", skip(app), err)]
async fn scenario_s12_closed_account(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (_, deposit_account_id) =
        helpers::create_customer(sub, app, "-nrsf03-s12-closed-account").await?;

    let amount = UsdCents::try_from_usd(dec!(100_000))?;
    app.deposits()
        .record_deposit(sub, deposit_account_id, amount, None)
        .await?;

    let withdrawal = app
        .deposits()
        .initiate_withdrawal(sub, deposit_account_id, amount, None)
        .await?;

    helpers::approve_and_confirm_withdrawal(sub, app, withdrawal.id).await?;

    app.deposits()
        .close_account(sub, deposit_account_id)
        .await?;

    event!(
        tracing::Level::INFO,
        "S12: Account closed after full withdrawal"
    );
    Ok(())
}

/// S14: Multiple deposit accounts per customer
/// One customer with two accounts, each with deposits
#[instrument(name = "sim_bootstrap.deposit_scenarios.s14", skip(app), err)]
async fn scenario_s14_multiple_accounts(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (customer_id, first_account_id) =
        helpers::create_customer(sub, app, "-nrsf03-s14-multiple-accounts").await?;

    let second_account = app.deposits().create_account(sub, customer_id).await?;

    let deposit_1_amount = UsdCents::try_from_usd(dec!(30_000))?;
    app.deposits()
        .record_deposit(sub, first_account_id, deposit_1_amount, None)
        .await?;

    let deposit_2_amount = UsdCents::try_from_usd(dec!(150_000))?;
    app.deposits()
        .record_deposit(sub, second_account.id, deposit_2_amount, None)
        .await?;

    event!(
        tracing::Level::INFO,
        "S14: Two accounts with deposits created"
    );
    Ok(())
}

/// S16: Zero balance account
/// Create an account with no deposits
#[instrument(name = "sim_bootstrap.deposit_scenarios.s16", skip(app), err)]
async fn scenario_s16_zero_balance(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (_, _) = helpers::create_customer(sub, app, "-nrsf03-s16-zero-balance").await?;

    event!(tracing::Level::INFO, "S16: Zero balance account created");
    Ok(())
}

/// S18: Customer with deposit only, no credit facility
#[instrument(name = "sim_bootstrap.deposit_scenarios.s18", skip(app), err)]
async fn scenario_s18_deposit_only(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (_, deposit_account_id) =
        helpers::create_customer(sub, app, "-nrsf03-s18-deposit-only").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(50_000))?;
    app.deposits()
        .record_deposit(sub, deposit_account_id, deposit_amount, None)
        .await?;

    event!(
        tracing::Level::INFO,
        "S18: Deposit-only customer created (no credit facility)"
    );
    Ok(())
}

/// S19: Many transactions
/// Multiple deposits and withdrawals to generate transaction volume
#[instrument(name = "sim_bootstrap.deposit_scenarios.s19", skip(app), err)]
async fn scenario_s19_many_transactions(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (_, deposit_account_id) =
        helpers::create_customer(sub, app, "-nrsf03-s19-many-transactions").await?;

    // 4 deposits
    for amount in [dec!(10_000), dec!(20_000), dec!(15_000), dec!(5_000)] {
        let usd_cents = UsdCents::try_from_usd(amount)?;
        app.deposits()
            .record_deposit(sub, deposit_account_id, usd_cents, None)
            .await?;
    }

    // 2 withdrawals
    for amount in [dec!(8_000), dec!(12_000)] {
        let usd_cents = UsdCents::try_from_usd(amount)?;
        let withdrawal = app
            .deposits()
            .initiate_withdrawal(sub, deposit_account_id, usd_cents, None)
            .await?;

        helpers::approve_and_confirm_withdrawal(sub, app, withdrawal.id).await?;
    }

    event!(
        tracing::Level::INFO,
        "S19: Multiple transactions completed (4 deposits, 2 withdrawals)"
    );
    Ok(())
}
