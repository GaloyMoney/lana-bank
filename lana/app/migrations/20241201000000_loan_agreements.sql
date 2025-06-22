-- Create loan agreements table
CREATE TABLE loan_agreements (
  id UUID PRIMARY KEY,
  customer_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create loan agreement events table
CREATE TABLE loan_agreement_events (
  id UUID NOT NULL REFERENCES loan_agreements(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);