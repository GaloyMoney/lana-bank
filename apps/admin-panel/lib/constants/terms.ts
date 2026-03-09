import { InterestInterval, Period, DisbursalPolicy } from "../graphql/generated"

export const TERMS_FIELD_LIMITS = {
  annualRate: { min: 0, max: 50 },
  durationUnits: { min: 1, max: 120 },
  oneTimeFeeRate: { min: 0, max: 10 },
  initialCvl: { min: 100, max: 500 },
  marginCallCvl: { min: 100, max: 500 },
  liquidationCvl: { min: 100, max: 500 },
} as const

export type TermsFieldsToValidate = {
  annualRate: string
  durationUnits: string
  oneTimeFeeRate: string
  initialCvl: string
  marginCallCvl: string
  liquidationCvl: string
}

export function validateTermsFields(values: TermsFieldsToValidate): string | null {
  const annualRate = parseFloat(values.annualRate)
  const durationUnits = parseFloat(values.durationUnits)
  const oneTimeFeeRate = parseFloat(values.oneTimeFeeRate)
  const initialCvl = parseFloat(values.initialCvl)
  const marginCallCvl = parseFloat(values.marginCallCvl)
  const liquidationCvl = parseFloat(values.liquidationCvl)

  if (isNaN(annualRate) || annualRate < TERMS_FIELD_LIMITS.annualRate.min || annualRate > TERMS_FIELD_LIMITS.annualRate.max)
    return "annualRateRange"
  if (isNaN(durationUnits) || durationUnits < TERMS_FIELD_LIMITS.durationUnits.min || durationUnits > TERMS_FIELD_LIMITS.durationUnits.max)
    return "durationRange"
  if (isNaN(oneTimeFeeRate) || oneTimeFeeRate < TERMS_FIELD_LIMITS.oneTimeFeeRate.min || oneTimeFeeRate > TERMS_FIELD_LIMITS.oneTimeFeeRate.max)
    return "oneTimeFeeRateRange"
  if (isNaN(initialCvl) || initialCvl < TERMS_FIELD_LIMITS.initialCvl.min || initialCvl > TERMS_FIELD_LIMITS.initialCvl.max)
    return "initialCvlRange"
  if (isNaN(marginCallCvl) || marginCallCvl < TERMS_FIELD_LIMITS.marginCallCvl.min || marginCallCvl > TERMS_FIELD_LIMITS.marginCallCvl.max)
    return "marginCallCvlRange"
  if (isNaN(liquidationCvl) || liquidationCvl < TERMS_FIELD_LIMITS.liquidationCvl.min || liquidationCvl > TERMS_FIELD_LIMITS.liquidationCvl.max)
    return "liquidationCvlRange"
  if (initialCvl <= marginCallCvl || marginCallCvl <= liquidationCvl)
    return "cvlOrderError"

  return null
}

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
  DISBURSAL_POLICY: DisbursalPolicy.SingleDisbursal,
}
