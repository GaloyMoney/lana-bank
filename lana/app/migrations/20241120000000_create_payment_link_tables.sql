-- Payment Link Tables
-- These tables support the FundingLink entity which manages the relationship
-- between credit facilities and deposit accounts for disbursement purposes.

CREATE TABLE cpl_funding_links (
    id UUID PRIMARY KEY,
    customer_id UUID NOT NULL,
    deposit_account_id UUID NOT NULL,
    credit_facility_id UUID NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cpl_funding_links_customer_id ON cpl_funding_links(customer_id);
CREATE INDEX idx_cpl_funding_links_deposit_account_id ON cpl_funding_links(deposit_account_id);
CREATE INDEX idx_cpl_funding_links_credit_facility_id ON cpl_funding_links(credit_facility_id);
CREATE INDEX idx_cpl_funding_links_status ON cpl_funding_links(status);

CREATE TABLE cpl_funding_link_events (
    id UUID NOT NULL REFERENCES cpl_funding_links(id),
    sequence INT NOT NULL,
    event_type VARCHAR NOT NULL,
    event JSONB NOT NULL,
    context JSONB DEFAULT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, sequence)
);

CREATE INDEX idx_cpl_funding_link_events_recorded_at ON cpl_funding_link_events(recorded_at);

