import { Period } from "../graphql/generated"

export const DEFAULT_TERMS = {
  OBLIGATION_OVERDUE_DURATION: {
    UNITS: 85,
    PERIOD: Period.Days,
  },
  INTEREST_DUE_DURATION: {
    UNITS: 0,
    PERIOD: Period.Days,
  },
}
