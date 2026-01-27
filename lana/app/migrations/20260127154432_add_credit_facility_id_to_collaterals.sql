-- Add credit_facility_id column to core_collaterals for efficient lookups

-- Step 1: Add the column as nullable initially
ALTER TABLE core_collaterals ADD COLUMN credit_facility_id UUID;

-- Step 2: Backfill from the events table (credit_facility_id is in Initialized event)
UPDATE core_collaterals c
SET credit_facility_id = (
    SELECT (e.event ->> 'credit_facility_id')::UUID
    FROM core_collateral_events e
    WHERE e.id = c.id
      AND e.event_type = 'initialized'
    LIMIT 1
)
WHERE c.credit_facility_id IS NULL;

-- Step 3: Make the column NOT NULL after backfill
ALTER TABLE core_collaterals ALTER COLUMN credit_facility_id SET NOT NULL;

-- Step 4: Add foreign key constraint
ALTER TABLE core_collaterals
ADD CONSTRAINT fk_core_collaterals_credit_facility_id
FOREIGN KEY (credit_facility_id) REFERENCES core_credit_facilities(id);

-- Step 5: Create index for lookups
CREATE INDEX idx_core_collaterals_credit_facility_id ON core_collaterals(credit_facility_id);
