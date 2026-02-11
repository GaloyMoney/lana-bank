import {
  DepositModuleConfig,
  DepositModuleConfigureInput,
} from "@/lib/graphql/generated"

export type DepositAccountCategoryKey = "asset" | "liability"

export type DepositConfigField = {
  key: keyof DepositModuleConfigureInput
  defaultCode: string
  category: DepositAccountCategoryKey
  group: "omnibus" | "summary"
}

export type DepositChangeItem = {
  field: DepositConfigField
  from: string
  to: string
}

export const DEPOSIT_CONFIG_FIELDS: DepositConfigField[] = [
  {
    key: "chartOfAccountsOmnibusParentCode",
    defaultCode: "1110.01.0101",
    category: "asset",
    group: "omnibus",
  },
  {
    key: "chartOfAccountsIndividualDepositAccountsParentCode",
    defaultCode: "2110.01.0401",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountsGovernmentEntityDepositAccountsParentCode",
    defaultCode: "2110.01.0201",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountPrivateCompanyDepositAccountsParentCode",
    defaultCode: "2110.01.0301",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountBankDepositAccountsParentCode",
    defaultCode: "2110.01.0501",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFinancialInstitutionDepositAccountsParentCode",
    defaultCode: "2110.01.0601",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountNonDomiciledIndividualDepositAccountsParentCode",
    defaultCode: "2110.01.0901",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountsFrozenIndividualDepositAccountsParentCode",
    defaultCode: "2114.03.0401",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode",
    defaultCode: "2114.03.0201",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode",
    defaultCode: "2114.03.0301",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFrozenBankDepositAccountsParentCode",
    defaultCode: "2114.03.0501",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode",
    defaultCode: "2114.03.0601",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode",
    defaultCode: "2114.03.0701",
    category: "liability",
    group: "summary",
  },
]

export const DEPOSIT_FIELD_GROUPS: Array<{
  key: DepositConfigField["group"]
  titleKey: "omnibus" | "summary"
}> = [
  { key: "omnibus", titleKey: "omnibus" },
  { key: "summary", titleKey: "summary" },
]

const buildFormData = (
  valueForField: (field: DepositConfigField) => string,
): DepositModuleConfigureInput =>
  DEPOSIT_CONFIG_FIELDS.reduce(
    (acc, field) => {
      acc[field.key] = valueForField(field)
      return acc
    },
    {} as DepositModuleConfigureInput,
  )

export const DEPOSIT_DEFAULT_FORM_DATA = buildFormData((field) => field.defaultCode)

export const DEPOSIT_EMPTY_FORM_DATA = buildFormData(() => "")

export const buildDepositFormDataFromConfig = (
  depositModuleConfig?: DepositModuleConfig,
): DepositModuleConfigureInput => {
  const updatedFormData = { ...DEPOSIT_EMPTY_FORM_DATA }
  if (!depositModuleConfig) return updatedFormData

  DEPOSIT_CONFIG_FIELDS.forEach((field) => {
    const value = depositModuleConfig[field.key as keyof DepositModuleConfig]
    if (value) {
      updatedFormData[field.key] = value as string
    }
  })

  return updatedFormData
}

export const buildDepositChanges = (
  baseline: DepositModuleConfigureInput,
  current: DepositModuleConfigureInput,
): DepositChangeItem[] =>
  DEPOSIT_CONFIG_FIELDS.flatMap((field) => {
    const from = baseline[field.key] ?? ""
    const to = current[field.key] ?? ""
    if (from === to) return []
    return [{ field, from, to }]
  })
