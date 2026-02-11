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
  defaultCode: string
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
    defaultCode: "9110.02.0201",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountCollateralOmnibusParentCode",
    defaultCode: "9220.08.0201",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountLiquidationProceedsOmnibusParentCode",
    defaultCode: "9170.00.0001",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountPaymentsMadeOmnibusParentCode",
    defaultCode: "9110.02.0201",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountInterestAddedToObligationsOmnibusParentCode",
    defaultCode: "9110.02.0201",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountFacilityParentCode",
    defaultCode: "9110.02.0201",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountCollateralParentCode",
    defaultCode: "9220.08.0201",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountCollateralInLiquidationParentCode",
    defaultCode: "9220.08.0201",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountLiquidatedCollateralParentCode",
    defaultCode: "9220.08.0201",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountProceedsFromLiquidationParentCode",
    defaultCode: "9220.08.0201",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountInterestIncomeParentCode",
    defaultCode: "6110.01.0100",
    category: "revenue",
    group: "summary",
  },
  {
    key: "chartOfAccountFeeIncomeParentCode",
    defaultCode: "6110.01.0300",
    category: "revenue",
    group: "summary",
  },
  {
    key: "chartOfAccountPaymentHoldingParentCode",
    defaultCode: "1141.99.0201",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountUncoveredOutstandingParentCode",
    defaultCode: "9110.02.0201",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountDisbursedDefaultedParentCode",
    defaultCode: "1148.04.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountInterestDefaultedParentCode",
    defaultCode: "1148.04.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermIndividualDisbursedReceivableParentCode",
    defaultCode: "1141.04.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode",
    defaultCode: "1141.02.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode",
    defaultCode: "1141.03.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermBankDisbursedReceivableParentCode",
    defaultCode: "1141.05.0401",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode",
    defaultCode: "1141.06.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode",
    defaultCode: "1141.07.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode",
    defaultCode: "1141.08.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermIndividualDisbursedReceivableParentCode",
    defaultCode: "1142.04.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode",
    defaultCode: "1142.02.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode",
    defaultCode: "1142.03.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermBankDisbursedReceivableParentCode",
    defaultCode: "1142.05.0401",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode",
    defaultCode: "1142.06.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode",
    defaultCode: "1142.07.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode",
    defaultCode: "1142.08.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermIndividualInterestReceivableParentCode",
    defaultCode: "1141.04.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode",
    defaultCode: "1141.02.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode",
    defaultCode: "1141.03.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermBankInterestReceivableParentCode",
    defaultCode: "1141.05.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode",
    defaultCode: "1141.06.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode",
    defaultCode: "1141.07.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode",
    defaultCode: "1141.08.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermIndividualInterestReceivableParentCode",
    defaultCode: "1142.04.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode",
    defaultCode: "1142.02.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode",
    defaultCode: "1142.03.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermBankInterestReceivableParentCode",
    defaultCode: "1142.05.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode",
    defaultCode: "1142.06.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode",
    defaultCode: "1142.07.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode",
    defaultCode: "1142.08.9901",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueIndividualDisbursedReceivableParentCode",
    defaultCode: "1148.04.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode",
    defaultCode: "1148.02.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode",
    defaultCode: "1148.03.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueBankDisbursedReceivableParentCode",
    defaultCode: "1148.05.0401",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode",
    defaultCode: "1148.06.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode",
    defaultCode: "1148.07.0101",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode",
    defaultCode: "1148.08.0101",
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

export const CREDIT_DEFAULT_FORM_DATA = buildFormData((field) => field.defaultCode)

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
