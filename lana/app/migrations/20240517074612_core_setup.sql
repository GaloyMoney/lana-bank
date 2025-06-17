CREATE TABLE committees (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE committee_events (
  id UUID NOT NULL REFERENCES committees(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE policies (
  id UUID PRIMARY KEY,
  committee_id UUID REFERENCES committees(id),
  process_type VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE policy_events (
  id UUID NOT NULL REFERENCES policies(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE approval_processes (
  id UUID PRIMARY KEY,
  policy_id UUID REFERENCES policies(id),
  committee_id UUID REFERENCES committees(id),
  process_type VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE approval_process_events (
  id UUID NOT NULL REFERENCES approval_processes(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
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
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_deposit_accounts (
  id UUID PRIMARY KEY,
  account_holder_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_deposit_account_events (
  id UUID NOT NULL REFERENCES core_deposit_accounts(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_deposits (
  id UUID PRIMARY KEY,
  deposit_account_id UUID NOT NULL REFERENCES core_deposit_accounts(id),
  reference VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_deposit_events (
  id UUID NOT NULL REFERENCES core_deposits(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_withdrawals (
  id UUID PRIMARY KEY,
  deposit_account_id UUID NOT NULL REFERENCES core_deposit_accounts(id),
  approval_process_id UUID REFERENCES approval_processes(id),
  cancelled_tx_id UUID DEFAULT NULL,
  reference VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_withdrawal_events (
  id UUID NOT NULL REFERENCES core_withdrawals(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE customers (
  id UUID PRIMARY KEY,
  authentication_id UUID UNIQUE DEFAULT NULL,
  email VARCHAR NOT NULL UNIQUE,
  telegram_id VARCHAR NOT NULL UNIQUE,
  status VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE customer_events (
  id UUID NOT NULL REFERENCES customers(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE terms_templates (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE terms_template_events (
  id UUID NOT NULL REFERENCES terms_templates(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
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
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_users (
  id UUID PRIMARY KEY,
  email VARCHAR NOT NULL UNIQUE,
  authentication_id UUID UNIQUE DEFAULT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_user_events (
  id UUID NOT NULL REFERENCES core_users(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_collaterals (
  id UUID PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_collateral_events (
  id UUID NOT NULL REFERENCES core_collaterals(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_credit_facilities (
  id UUID PRIMARY KEY,
  customer_id UUID NOT NULL REFERENCES customers(id),
  approval_process_id UUID NOT NULL REFERENCES approval_processes(id),
  collateralization_ratio NUMERIC,
  collateralization_state VARCHAR NOT NULL,
  status VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_credit_facility_events (
  id UUID NOT NULL REFERENCES core_credit_facilities(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_custodians (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_custodian_events (
  id UUID NOT NULL REFERENCES core_custodians(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
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
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE core_disbursals (
  id UUID PRIMARY KEY,
  credit_facility_id UUID NOT NULL REFERENCES core_credit_facilities(id),
  approval_process_id UUID NOT NULL REFERENCES approval_processes(id),
  obligation_id UUID DEFAULT NULL REFERENCES core_obligations(id),
  concluded_tx_id UUID DEFAULT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_disbursal_events (
  id UUID NOT NULL REFERENCES core_disbursals(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
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
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

-- Document storage tables
CREATE TABLE core_documents (
  id UUID PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE core_document_events (
  id UUID NOT NULL REFERENCES core_documents(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE documents (
  id UUID PRIMARY KEY,
  deleted BOOLEAN NOT NULL DEFAULT FALSE,
  customer_id UUID NOT NULL REFERENCES customers(id),
  created_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_documents_customer_id_deleted_id ON documents (customer_id, deleted, id);

CREATE TABLE document_events (
  id UUID NOT NULL REFERENCES documents(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
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
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TABLE accounting_csvs (
  id UUID PRIMARY KEY,
  csv_type VARCHAR NOT NULL,
  ledger_account_id UUID,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE accounting_csv_events (
  id UUID NOT NULL REFERENCES accounting_csvs(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
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
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE TYPE JobExecutionState AS ENUM ('pending', 'running');

CREATE TABLE job_executions (
  id UUID REFERENCES jobs(id) NOT NULL UNIQUE,
  attempt_index INT NOT NULL DEFAULT 1,
  state JobExecutionState NOT NULL DEFAULT 'pending',
  execution_state_json JSONB,
  reschedule_after TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

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
  id UUID PRIMARY KEY REFERENCES core_credit_facilities(id),
  history JSONB NOT NULL DEFAULT '[]',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE core_credit_facility_repayment_plans (
  id UUID PRIMARY KEY REFERENCES core_credit_facilities(id),
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

