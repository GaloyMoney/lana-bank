-- Add collateral_id column to core_liquidations for nested entity relationship
-- This enables Liquidation to be a nested entity under Collateral

-- Step 1: Add the column as nullable initially
ALTER TABLE core_liquidations ADD COLUMN collateral_id UUID;

-- Step 2: Backfill from the events table
-- The collateral_id is stored in the Initialized event
UPDATE core_liquidations l
SET collateral_id = (
    SELECT (e.event ->> 'collateral_id')::UUID
    FROM core_liquidation_events e
    WHERE e.id = l.id
      AND e.event_type = 'initialized'
    LIMIT 1
)
WHERE l.collateral_id IS NULL;

-- Step 3: Make the column NOT NULL after backfill
ALTER TABLE core_liquidations ALTER COLUMN collateral_id SET NOT NULL;

-- Step 4: Add foreign key constraint
ALTER TABLE core_liquidations
ADD CONSTRAINT fk_core_liquidations_collateral_id
FOREIGN KEY (collateral_id) REFERENCES core_collaterals(id);

-- Step 5: Create index for parent lookup (nested entity pattern)
CREATE INDEX idx_core_liquidations_collateral_id ON core_liquidations(collateral_id);
