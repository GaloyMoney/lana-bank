import { InterestInterval, Period } from "../graphql/generated"

export const DEFAULT_TERMS = {
  OBLIGATION_OVERDUE_DURATION: {
    UNITS: 85,
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
