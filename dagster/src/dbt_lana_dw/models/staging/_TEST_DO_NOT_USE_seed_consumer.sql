-- TEST MODEL: Delete this file when removing the test seed
-- This model exists only to verify seed-to-model dependency mapping in Dagster
select id, name, description, 'consumed_from_seed' as source_type
from {{ ref("_TEST_DO_NOT_USE_example_seed") }}
