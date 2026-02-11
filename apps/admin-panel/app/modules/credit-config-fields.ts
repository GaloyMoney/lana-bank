import {
  CreditModuleConfig,
  CreditModuleConfigureInput,
} from "@/lib/graphql/generated"

export type CreditAccountCategoryKey =
  | "offBalanceSheet"
  | "asset"
  | "liability"
  | "equity"
  | "revenue"
  | "costOfRevenue"
  | "expenses"

export type CreditConfigField = {
  key: keyof CreditModuleConfigureInput
  category: CreditAccountCategoryKey
  group: "omnibus" | "summary"
}

export type CreditChangeItem = {
  field: CreditConfigField
  from: string
  to: string
}

export const CREDIT_CONFIG_FIELDS: CreditConfigField[] = [
  {
    key: "chartOfAccountFacilityOmnibusParentCode",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountCollateralOmnibusParentCode",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountLiquidationProceedsOmnibusParentCode",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountPaymentsMadeOmnibusParentCode",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountInterestAddedToObligationsOmnibusParentCode",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountFacilityParentCode",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountCollateralParentCode",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountCollateralInLiquidationParentCode",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountLiquidatedCollateralParentCode",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountProceedsFromLiquidationParentCode",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountInterestIncomeParentCode",
    category: "revenue",
    group: "summary",
  },
  {
    key: "chartOfAccountFeeIncomeParentCode",
    category: "revenue",
    group: "summary",
  },
  {
    key: "chartOfAccountPaymentHoldingParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountUncoveredOutstandingParentCode",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountDisbursedDefaultedParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountInterestDefaultedParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermIndividualDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermBankDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermIndividualDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermBankDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermIndividualInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermBankInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermIndividualInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermBankInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueIndividualDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueBankDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode",
    category: "asset",
    group: "summary",
  },
]

export const CREDIT_FIELD_GROUPS: Array<{
  key: CreditConfigField["group"]
  titleKey: "omnibus" | "summary"
}> = [
  { key: "omnibus", titleKey: "omnibus" },
  { key: "summary", titleKey: "summary" },
]

const buildFormData = (
  valueForField: (field: CreditConfigField) => string,
): CreditModuleConfigureInput =>
  CREDIT_CONFIG_FIELDS.reduce(
    (acc, field) => {
      acc[field.key] = valueForField(field)
      return acc
    },
    {} as CreditModuleConfigureInput,
  )

export const CREDIT_EMPTY_FORM_DATA = buildFormData(() => "")

export const buildCreditFormDataFromConfig = (
  creditModuleConfig?: CreditModuleConfig,
): CreditModuleConfigureInput => {
  const updatedFormData = { ...CREDIT_EMPTY_FORM_DATA }
  if (!creditModuleConfig) return updatedFormData

  CREDIT_CONFIG_FIELDS.forEach((field) => {
    const value = creditModuleConfig[field.key as keyof CreditModuleConfig]
    if (value) {
      updatedFormData[field.key] = value as string
    }
  })

  return updatedFormData
}

export const buildCreditChanges = (
  baseline: CreditModuleConfigureInput,
  current: CreditModuleConfigureInput,
): CreditChangeItem[] =>
  CREDIT_CONFIG_FIELDS.flatMap((field) => {
    const from = baseline[field.key] ?? ""
    const to = current[field.key] ?? ""
    if (from === to) return []
    return [{ field, from, to }]
  })
