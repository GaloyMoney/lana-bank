CREATE TABLE customer_activity (
    customer_id UUID PRIMARY KEY REFERENCES core_customers(id),
    last_activity_date TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_customer_activity_last_activity_date ON customer_activity(last_activity_date);
