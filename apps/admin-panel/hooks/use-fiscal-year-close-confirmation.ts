import { useCallback, useMemo, useState, type Dispatch, type SetStateAction } from "react"

import { useLocale, useTranslations } from "next-intl"

import type { FiscalYear } from "@/lib/graphql/generated"
import {
  formatUTCMonthName,
  formatUTCMonthYear,
  getUTCYear,
} from "@/utils/fiscal-year-dates"

type FiscalYearInput = Pick<FiscalYear, "openedAsOf">
type FiscalMonthInput = Pick<FiscalYear, "nextMonthToClose">

type ConfirmationResult = {
  confirmationText: string | null
  displayText: string | null
  input: string
  setInput: Dispatch<SetStateAction<string>>
  isValid: boolean
  reset: () => void
}

function useConfirmationInput(confirmationText: string | null, locale: string) {
  const [input, setInput] = useState("")
  const reset = useCallback(() => setInput(""), [])
  const normalize = useCallback(
    (value: string) => value.trim().replace(/\s+/g, " ").toLocaleUpperCase(locale),
    [locale],
  )
  const isValid = confirmationText
    ? normalize(input) === normalize(confirmationText)
    : false
  return { input, setInput, isValid, reset }
}

export function useFiscalYearCloseConfirmation(
  fiscalYear: FiscalYearInput,
): ConfirmationResult {
  const locale = useLocale()
  const closeWord = useTranslations("Common")("close").toLocaleUpperCase(locale)

  const { confirmationText, displayText } = useMemo(() => {
    const year = getUTCYear(fiscalYear.openedAsOf)
    if (year === null) return { confirmationText: null, displayText: null }
    return {
      confirmationText: `${closeWord} ${year}`,
      displayText: year.toString(),
    }
  }, [closeWord, fiscalYear.openedAsOf])

  const confirmationState = useConfirmationInput(confirmationText, locale)
  return { confirmationText, displayText, ...confirmationState }
}

export function useFiscalMonthCloseConfirmation(
  fiscalYear: FiscalMonthInput,
): ConfirmationResult {
  const locale = useLocale()
  const closeWord = useTranslations("Common")("close").toLocaleUpperCase(locale)

  const { confirmationText, displayText } = useMemo(() => {
    if (!fiscalYear.nextMonthToClose) {
      return { confirmationText: null, displayText: null }
    }
    const monthName = formatUTCMonthName(fiscalYear.nextMonthToClose, locale)
    const monthYear = formatUTCMonthYear(fiscalYear.nextMonthToClose, locale)
    if (!monthName || !monthYear) return { confirmationText: null, displayText: null }

    return {
      confirmationText: `${closeWord} ${monthName.toLocaleUpperCase(locale)}`,
      displayText: monthYear,
    }
  }, [closeWord, fiscalYear.nextMonthToClose, locale])
  const confirmationState = useConfirmationInput(confirmationText, locale)
  return { confirmationText, displayText, ...confirmationState }
}
