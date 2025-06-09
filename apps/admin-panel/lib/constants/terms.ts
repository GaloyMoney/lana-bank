import { InterestInterval, Period } from "../graphql/generated"

export const DEFAULT_TERMS = {
  OBLIGATION_LIQUIDATION_DURATION_FROM_DUE: {
    UNITS: 60,
    PERIOD: Period.Days,
  },
  OBLIGATION_OVERDUE_DURATION_FROM_DUE: {
    UNITS: 50,
    PERIOD: Period.Days,
  },
  INTEREST_DUE_DURATION_FROM_ACCRUAL: {
    UNITS: 0,
    PERIOD: Period.Days,
  },
  ACCRUAL_CYCLE_INTERVAL: InterestInterval.EndOfMonth,
  ACCRUAL_INTERVAL: InterestInterval.EndOfDay,
  DURATION_PERIOD: Period.Months,
}
