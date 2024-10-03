CREATE TABLE customers (
  id UUID PRIMARY KEY,
  email VARCHAR NOT NULL UNIQUE,
  telegram_id VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_customers_email_id ON customers (email, id);

CREATE TABLE customer_events (
  id UUID NOT NULL REFERENCES customers(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE terms_templates (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE terms_template_events (
  id UUID NOT NULL REFERENCES terms_templates(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE users (
  id UUID PRIMARY KEY,
  email VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user_events (
  id UUID NOT NULL REFERENCES users(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE loans (
  id UUID PRIMARY KEY,
  customer_id UUID NOT NULL REFERENCES customers(id),
  collateralization_ratio NUMERIC,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_loans_collateralization_ratio ON loans (collateralization_ratio);

CREATE TABLE loan_events (
  id UUID NOT NULL REFERENCES loans(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE credit_facilities (
  id UUID PRIMARY KEY,
  customer_id UUID NOT NULL REFERENCES customers(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE credit_facility_events (
  id UUID NOT NULL REFERENCES credit_facilities(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE disbursements (
  id UUID PRIMARY KEY,
  credit_facility_id UUID NOT NULL REFERENCES credit_facilities(id),
  idx INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(credit_facility_id, idx)
);

CREATE TABLE disbursement_events (
  id UUID NOT NULL REFERENCES disbursements(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE withdraws (
  id UUID PRIMARY KEY,
  customer_id UUID NOT NULL REFERENCES customers(id),
  reference VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE withdraw_events (
  id UUID NOT NULL REFERENCES withdraws(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE deposits (
  id UUID PRIMARY KEY,
  customer_id UUID NOT NULL REFERENCES customers(id),
  reference VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE deposit_events (
  id UUID NOT NULL REFERENCES deposits(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE reports (
  id UUID PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE report_events (
  id UUID NOT NULL REFERENCES reports(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TABLE jobs (
  id UUID NOT NULL UNIQUE,
  name VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_job_name ON jobs(name);

CREATE TABLE job_events (
  id UUID REFERENCES jobs(id) NOT NULL,
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);

CREATE TYPE JobExecutionState AS ENUM ('pending', 'running');

CREATE TABLE job_executions (
  id UUID REFERENCES jobs(id) NOT NULL UNIQUE,
  state JobExecutionState NOT NULL DEFAULT 'pending',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  reschedule_after TIMESTAMPTZ NOT NULL DEFAULT NOW()
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

CREATE TABLE sumsub_callbacks (
  id BIGSERIAL PRIMARY KEY,
  customer_id UUID NOT NULL REFERENCES customers(id),
  content JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
