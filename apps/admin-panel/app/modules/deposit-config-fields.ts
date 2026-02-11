import {
  DepositModuleConfig,
  DepositModuleConfigureInput,
} from "@/lib/graphql/generated"

export type DepositAccountCategoryKey = "asset" | "liability"

export type DepositConfigField = {
  key: keyof DepositModuleConfigureInput
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
    category: "asset",
    group: "omnibus",
  },
  {
    key: "chartOfAccountsIndividualDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountsGovernmentEntityDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountPrivateCompanyDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountBankDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFinancialInstitutionDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountNonDomiciledIndividualDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountsFrozenIndividualDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFrozenBankDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode",
    category: "liability",
    group: "summary",
  },
  {
    key: "chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode",
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
