import dagster as dg

COLD_START_CONDITION = (
    dg.AutomationCondition.missing()
    & ~dg.AutomationCondition.in_progress()
    & dg.AutomationCondition.in_latest_time_window()
    & ~dg.AutomationCondition.any_deps_missing()
)

# Same, as above, but without the deps guard. Use for assets whose upstream dependencies
# are external sources that Dagster considers permanently "missing".
COLD_START_CONDITION_SKIP_DEPS = (
    dg.AutomationCondition.missing()
    & ~dg.AutomationCondition.in_progress()
    & dg.AutomationCondition.in_latest_time_window()
)
