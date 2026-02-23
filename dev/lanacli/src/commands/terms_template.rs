use anyhow::Result;

use crate::cli::TermsTemplateAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output::{self, scalar, sval};

pub async fn execute(
    client: &mut GraphQLClient,
    action: TermsTemplateAction,
    json: bool,
) -> Result<()> {
    match action {
        TermsTemplateAction::Create {
            name,
            annual_rate,
            accrual_interval,
            accrual_cycle_interval,
            one_time_fee_rate,
            disbursal_policy,
            duration_months,
            initial_cvl,
            margin_call_cvl,
            liquidation_cvl,
            interest_due_days,
            overdue_days,
            liquidation_days,
        } => {
            let vars = terms_template_create::Variables {
                input: terms_template_create::TermsTemplateCreateInput {
                    name,
                    annual_rate: sval(annual_rate),
                    accrual_interval: parse_interest_interval(&accrual_interval)?,
                    accrual_cycle_interval: parse_interest_interval(&accrual_cycle_interval)?,
                    one_time_fee_rate: sval(one_time_fee_rate),
                    disbursal_policy: parse_disbursal_policy(&disbursal_policy)?,
                    duration: terms_template_create::DurationInput {
                        period: terms_template_create::Period::MONTHS,
                        units: duration_months,
                    },
                    initial_cvl: sval(initial_cvl),
                    margin_call_cvl: sval(margin_call_cvl),
                    liquidation_cvl: sval(liquidation_cvl),
                    interest_due_duration_from_accrual: terms_template_create::DurationInput {
                        period: terms_template_create::Period::DAYS,
                        units: interest_due_days,
                    },
                    obligation_overdue_duration_from_due: terms_template_create::DurationInput {
                        period: terms_template_create::Period::DAYS,
                        units: overdue_days,
                    },
                    obligation_liquidation_duration_from_due:
                        terms_template_create::DurationInput {
                            period: terms_template_create::Period::DAYS,
                            units: liquidation_days,
                        },
                },
            };
            let data = client.execute::<TermsTemplateCreate>(vars).await?;
            let t = data.terms_template_create.terms_template;
            if json {
                output::print_json(&t)?;
            } else {
                let annual_rate = scalar(&t.values.annual_rate);
                let fee_rate = scalar(&t.values.one_time_fee_rate);
                output::print_kv(&[
                    ("Terms ID", &t.terms_id),
                    ("Name", &t.name),
                    ("Annual Rate", &annual_rate),
                    (
                        "Accrual Interval",
                        &format!("{:?}", t.values.accrual_interval),
                    ),
                    ("One-time Fee", &fee_rate),
                    (
                        "Disbursal Policy",
                        &format!("{:?}", t.values.disbursal_policy),
                    ),
                    (
                        "Duration",
                        &format!("{} {:?}", t.values.duration.units, t.values.duration.period),
                    ),
                    ("Created", &t.created_at),
                ]);
            }
        }
        TermsTemplateAction::List => {
            let vars = terms_templates_list::Variables;
            let data = client.execute::<TermsTemplatesList>(vars).await?;
            let templates = data.terms_templates;
            if json {
                output::print_json(&templates)?;
            } else {
                let rows: Vec<Vec<String>> = templates
                    .iter()
                    .map(|t| {
                        vec![
                            t.terms_id.clone(),
                            t.name.clone(),
                            scalar(&t.values.annual_rate),
                            format!("{:?}", t.values.disbursal_policy),
                            format!("{} {:?}", t.values.duration.units, t.values.duration.period),
                            t.created_at.clone(),
                        ]
                    })
                    .collect();
                output::print_table(
                    &[
                        "ID",
                        "Name",
                        "Annual Rate",
                        "Disbursal",
                        "Duration",
                        "Created",
                    ],
                    rows,
                );
            }
        }
    }
    Ok(())
}

fn parse_interest_interval(s: &str) -> Result<terms_template_create::InterestInterval> {
    match s.to_uppercase().as_str() {
        "END_OF_MONTH" => Ok(terms_template_create::InterestInterval::END_OF_MONTH),
        "END_OF_DAY" => Ok(terms_template_create::InterestInterval::END_OF_DAY),
        other => anyhow::bail!("Unknown interest interval: {other}"),
    }
}

fn parse_disbursal_policy(s: &str) -> Result<terms_template_create::DisbursalPolicy> {
    match s.to_uppercase().as_str() {
        "SINGLE_DISBURSAL" => Ok(terms_template_create::DisbursalPolicy::SINGLE_DISBURSAL),
        "MULTIPLE_DISBURSAL" => Ok(terms_template_create::DisbursalPolicy::MULTIPLE_DISBURSAL),
        other => anyhow::bail!("Unknown disbursal policy: {other}"),
    }
}
