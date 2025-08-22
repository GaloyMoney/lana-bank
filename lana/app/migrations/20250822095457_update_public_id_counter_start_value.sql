-- Update public_id counter to start at 1000 instead of 1
-- This ensures all new public_ids will start from 1000

ALTER SEQUENCE core_public_id_counter RESTART WITH 1000;
