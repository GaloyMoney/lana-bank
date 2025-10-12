CREATE TABLE core_committees (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_committee_events (
  id UUID NOT NULL REFERENCES core_committees(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_policies (
  id UUID PRIMARY KEY,
  committee_id UUID REFERENCES core_committees(id),
  process_type VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_policy_events (
  id UUID NOT NULL REFERENCES core_policies(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_approval_processes (
  id UUID PRIMARY KEY,
  policy_id UUID REFERENCES core_policies(id),
  committee_id UUID REFERENCES core_committees(id),
  process_type VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_approval_process_events (
  id UUID NOT NULL REFERENCES core_approval_processes(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_charts (
  id UUID PRIMARY KEY,
  reference VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_chart_events (
  id UUID NOT NULL REFERENCES core_charts(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_chart_nodes (
    id UUID PRIMARY KEY,
    chart_id UUID NOT NULL REFERENCES core_charts(id),
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_chart_node_events (
    id UUID NOT NULL REFERENCES core_chart_nodes(id),
    sequence INT NOT NULL,
    event_type VARCHAR NOT NULL,
    event JSONB NOT NULL,
    context JSONB DEFAULT NULL,
    recorded_at TIMESTAMPTZ NOT NULL,
    UNIQUE(id, sequence)
);

CREATE TABLE core_public_ids (
  id VARCHAR PRIMARY KEY,
  target_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_public_id_events (
  id VARCHAR NOT NULL REFERENCES core_public_ids(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE SEQUENCE core_public_id_counter START 1000;

CREATE TABLE core_customers (
  id UUID PRIMARY KEY,
  email VARCHAR NOT NULL UNIQUE,
  telegram_id VARCHAR NOT NULL UNIQUE,
  kyc_verification VARCHAR NOT NULL,
  activity VARCHAR NOT NULL DEFAULT 'disabled',
  public_id VARCHAR NOT NULL REFERENCES core_public_ids(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_customer_events (
  id UUID NOT NULL REFERENCES core_customers(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE customer_activity (
  customer_id UUID PRIMARY KEY REFERENCES core_customers(id),
  last_activity_date TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_customer_activity_last_activity_date ON customer_activity(last_activity_date);

CREATE TABLE core_deposit_accounts (
  id UUID PRIMARY KEY,
  account_holder_id UUID NOT NULL,
  public_id VARCHAR NOT NULL REFERENCES core_public_ids(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_deposit_account_events (
  id UUID NOT NULL REFERENCES core_deposit_accounts(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_deposits (
  id UUID PRIMARY KEY,
  deposit_account_id UUID NOT NULL REFERENCES core_deposit_accounts(id),
  reference VARCHAR NOT NULL UNIQUE,
  public_id VARCHAR NOT NULL REFERENCES core_public_ids(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_deposit_events (
  id UUID NOT NULL REFERENCES core_deposits(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_withdrawals (
  id UUID PRIMARY KEY,
  deposit_account_id UUID NOT NULL REFERENCES core_deposit_accounts(id),
  approval_process_id UUID REFERENCES core_approval_processes(id),
  cancelled_tx_id UUID DEFAULT NULL,
  reference VARCHAR NOT NULL UNIQUE,
  public_id VARCHAR NOT NULL REFERENCES core_public_ids(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_withdrawal_events (
  id UUID NOT NULL REFERENCES core_withdrawals(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_terms_templates (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_terms_template_events (
  id UUID NOT NULL REFERENCES core_terms_templates(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_permission_sets (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_permission_set_events (
  id UUID NOT NULL REFERENCES core_permission_sets(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_roles (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_role_events (
  id UUID NOT NULL REFERENCES core_roles(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_users (
  id UUID PRIMARY KEY,
  email VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_user_events (
  id UUID NOT NULL REFERENCES core_users(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_wallets (
  id UUID PRIMARY KEY,
  external_wallet_id VARCHAR NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_wallet_events (
  id UUID NOT NULL REFERENCES core_wallets(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_collaterals (
  id UUID PRIMARY KEY,
  custody_wallet_id UUID,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_collateral_events (
  id UUID NOT NULL REFERENCES core_collaterals(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_credit_facility_proposals (
  id UUID PRIMARY KEY,
  customer_id UUID NOT NULL REFERENCES core_customers(id),
  approval_process_id UUID NOT NULL REFERENCES core_approval_processes(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_credit_facility_proposal_events (
  id UUID NOT NULL REFERENCES core_credit_facility_proposals(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_pending_credit_facilities (
  id UUID PRIMARY KEY,
  credit_facility_proposal_id UUID NOT NULL REFERENCES core_credit_facility_proposals(id),
  customer_id UUID NOT NULL REFERENCES core_customers(id),
  approval_process_id UUID NOT NULL REFERENCES core_approval_processes(id),
  collateral_id UUID NOT NULL REFERENCES core_collaterals(id),
  collateralization_ratio NUMERIC,
  collateralization_state VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_pending_credit_facility_events (
  id UUID NOT NULL REFERENCES core_pending_credit_facilities(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_credit_facilities (
  id UUID PRIMARY KEY,
  customer_id UUID NOT NULL REFERENCES core_customers(id),
  pending_credit_facility_id UUID NOT NULL REFERENCES core_pending_credit_facilities(id),
  collateral_id UUID NOT NULL REFERENCES core_collaterals(id),
  collateralization_ratio NUMERIC,
  collateralization_state VARCHAR NOT NULL,
  status VARCHAR NOT NULL,
  public_id VARCHAR NOT NULL REFERENCES core_public_ids(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_credit_facility_events (
  id UUID NOT NULL REFERENCES core_credit_facilities(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_custodians (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  provider VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_custodian_events (
  id UUID NOT NULL REFERENCES core_custodians(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_custodian_webhook_notifications (
  custodian_id UUID NULL REFERENCES core_custodians(id),
  uri VARCHAR NOT NULL,
  headers JSONB NOT NULL,
  payload JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE core_obligations (
  id UUID PRIMARY KEY,
  credit_facility_id UUID NOT NULL REFERENCES core_credit_facilities(id),
  reference VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_obligation_events (
  id UUID NOT NULL REFERENCES core_obligations(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_liquidation_processes (
  id UUID PRIMARY KEY,
  obligation_id UUID NOT NULL REFERENCES core_obligations(id),
  credit_facility_id UUID NOT NULL REFERENCES core_credit_facilities(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_liquidation_process_events (
  id UUID NOT NULL REFERENCES core_liquidation_processes(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_disbursals (
  id UUID PRIMARY KEY,
  credit_facility_id UUID NOT NULL REFERENCES core_credit_facilities(id),
  approval_process_id UUID NOT NULL REFERENCES core_approval_processes(id),
  obligation_id UUID DEFAULT NULL REFERENCES core_obligations(id),
  concluded_tx_id UUID DEFAULT NULL,
  public_id VARCHAR NOT NULL REFERENCES core_public_ids(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_disbursal_events (
  id UUID NOT NULL REFERENCES core_disbursals(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_interest_accrual_cycles (
  id UUID PRIMARY KEY,
  credit_facility_id UUID NOT NULL REFERENCES core_credit_facilities(id),
  idx INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  UNIQUE(credit_facility_id, idx)
);

CREATE TABLE core_interest_accrual_cycle_events (
  id UUID NOT NULL REFERENCES core_interest_accrual_cycles(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_payments (
  id UUID PRIMARY KEY,
  credit_facility_id UUID NOT NULL REFERENCES core_credit_facilities(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_payment_events (
  id UUID NOT NULL REFERENCES core_payments(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_payment_allocations (
  id UUID PRIMARY KEY,
  payment_id UUID NOT NULL REFERENCES core_payments(id),
  obligation_id UUID NOT NULL REFERENCES core_obligations(id),
  credit_facility_id UUID NOT NULL REFERENCES core_credit_facilities(id),
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_payment_allocation_events (
  id UUID NOT NULL REFERENCES core_payment_allocations(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_documents (
  id UUID PRIMARY KEY,
  reference_id UUID NOT NULL,
  deleted BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_core_documents_reference_id ON core_documents(reference_id);

CREATE TABLE core_document_events (
  id UUID NOT NULL REFERENCES core_documents(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE core_reports (
  id UUID PRIMARY KEY,
  external_id VARCHAR NOT NULL UNIQUE,
  run_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_core_reports_run_id ON core_reports(run_id);

CREATE TABLE core_report_events (
  id UUID NOT NULL REFERENCES core_reports(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_report_runs (
  id UUID PRIMARY KEY,
  external_id VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_report_run_events (
  id UUID NOT NULL REFERENCES core_report_runs(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE reports (
  id UUID PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE report_events (
  id UUID NOT NULL REFERENCES reports(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_manual_transactions (
  id UUID PRIMARY KEY,
  reference VARCHAR NOT NULL UNIQUE,
  ledger_transaction_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_manual_transaction_events (
  id UUID NOT NULL REFERENCES core_manual_transactions(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE jobs (
  id UUID NOT NULL UNIQUE,
  unique_per_type BOOLEAN NOT NULL,
  job_type VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);
CREATE UNIQUE INDEX idx_unique_job_type ON jobs (job_type) WHERE unique_per_type = TRUE;

CREATE TABLE job_events (
  id UUID NOT NULL REFERENCES jobs(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TYPE JobExecutionState AS ENUM ('pending', 'running');

CREATE TABLE job_executions (
  id UUID REFERENCES jobs(id) NOT NULL UNIQUE,
  job_type VARCHAR NOT NULL,
  attempt_index INT NOT NULL DEFAULT 1,
  state JobExecutionState NOT NULL DEFAULT 'pending',
  execution_state_json JSONB,
  execute_at TIMESTAMPTZ,
  alive_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE OR REPLACE FUNCTION notify_job_execution_insert() RETURNS TRIGGER AS $$
BEGIN
  PERFORM pg_notify('job_execution', '');
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION notify_job_execution_update() RETURNS TRIGGER AS $$
BEGIN
  IF NEW.execute_at IS DISTINCT FROM OLD.execute_at THEN
    PERFORM pg_notify('job_execution', '');
  END IF;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER job_executions_notify_insert_trigger
AFTER INSERT ON job_executions
FOR EACH STATEMENT
EXECUTE FUNCTION notify_job_execution_insert();

CREATE TRIGGER job_executions_notify_update_trigger
AFTER UPDATE ON job_executions
FOR EACH ROW
EXECUTE FUNCTION notify_job_execution_update();

CREATE TABLE casbin_rule (
  id SERIAL PRIMARY KEY,
  ptype VARCHAR NOT NULL,
  v0 VARCHAR NOT NULL,
  v1 VARCHAR NOT NULL,
  v2 VARCHAR NOT NULL,
  v3 VARCHAR NOT NULL,
  v4 VARCHAR NOT NULL,
  v5 VARCHAR NOT NULL,
  CONSTRAINT unique_key_sqlx_adapter UNIQUE(ptype, v0, v1, v2, v3, v4, v5)
);

CREATE TABLE audit_entries (
  id BIGSERIAL PRIMARY KEY,
  subject VARCHAR NOT NULL,
  object VARCHAR NOT NULL,
  action VARCHAR NOT NULL,
  authorized BOOLEAN NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE core_credit_facility_histories (
  id UUID PRIMARY KEY REFERENCES core_credit_facility_proposals(id),
  history JSONB NOT NULL DEFAULT '[]',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE core_credit_facility_repayment_plans (
  id UUID PRIMARY KEY REFERENCES core_credit_facility_proposals(id),
  repayment_plan JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE dashboards (
  id UUID PRIMARY KEY,
  dashboard_json JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE sumsub_callbacks (
  id BIGSERIAL PRIMARY KEY,
  customer_id UUID NOT NULL, -- not enforced to get all callbacks
  content JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_sumsub_callbacks_customer_id ON sumsub_callbacks(customer_id);

CREATE TABLE persistent_outbox_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  sequence BIGSERIAL UNIQUE,
  payload JSONB,
  tracing_context JSONB,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE FUNCTION notify_persistent_outbox_events() RETURNS TRIGGER AS $$
DECLARE
  payload TEXT;
  payload_size INTEGER;
BEGIN
  payload := row_to_json(NEW);
  payload_size := octet_length(payload);
  IF payload_size <= 8000 THEN
    PERFORM pg_notify('persistent_outbox_events', payload);
  ELSE
    RAISE NOTICE 'Payload too large for notification: % bytes', payload_size;
  END IF;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER persistent_outbox_events AFTER INSERT ON persistent_outbox_events
  FOR EACH ROW EXECUTE FUNCTION notify_persistent_outbox_events();

