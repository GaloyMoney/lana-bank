import { InterestInterval, Period } from "../graphql/generated"

export const DEFAULT_TERMS = {
  OBLIGATION_LIQUIDATION_FROM_DUE_AFTER_DAYS: {
    UNITS: 60,
    PERIOD: Period.Days,
  },
  OBLIGATION_OVERDUE_FROM_DUE_AFTER_DAYS: {
    UNITS: 50,
    PERIOD: Period.Days,
  },
  INTEREST_DUE_FROM_ACCRUAL_AFTER_DAYS: {
    UNITS: 0,
    PERIOD: Period.Days,
  },
  ACCRUAL_CYCLE_INTERVAL: InterestInterval.EndOfMonth,
  ACCRUAL_INTERVAL: InterestInterval.EndOfDay,
  DURATION_PERIOD: Period.Months,
}
