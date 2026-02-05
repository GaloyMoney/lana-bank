"use client"

import { gql } from "@apollo/client"
import { Button } from "@lana/web/ui/button"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@lana/web/ui/command"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Label } from "@lana/web/ui/label"
import { Popover, PopoverContent, PopoverTrigger } from "@lana/web/ui/popover"
import { Tooltip, TooltipContent, TooltipTrigger } from "@lana/web/ui/tooltip"
import { cn } from "@lana/web/utils"
import { CheckIcon, ChevronsUpDownIcon } from "lucide-react"
import { useTranslations } from "next-intl"
import { FormEvent, useEffect, useMemo, useRef, useState } from "react"
import { toast } from "sonner"

import {
  CreditConfigDocument,
  CreditModuleConfig,
  CreditModuleConfigureInput,
  useCreditModuleConfigureMutation,
} from "@/lib/graphql/generated"

gql`
  mutation CreditModuleConfigure($input: CreditModuleConfigureInput!) {
    creditModuleConfigure(input: $input) {
      creditConfig {
        chartOfAccountsId
      }
    }
  }
`

type CreditAccountCategoryKey =
  | "offBalanceSheet"
  | "asset"
  | "liability"
  | "equity"
  | "revenue"
  | "costOfRevenue"
  | "expenses"

type AccountSetOption = {
  code: string
  name: string
  ref?: string
  category: CreditAccountCategoryKey
}

type CreditConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  creditModuleConfig?: CreditModuleConfig
  accountSetOptions?: AccountSetOption[]
}

type CreditConfigField = {
  key: keyof CreditModuleConfigureInput
  defaultCode: string
  category: CreditAccountCategoryKey
  group: "omnibus" | "summary"
}

const CREDIT_CONFIG_FIELDS: CreditConfigField[] = [
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
    category: "revenue",
    group: "omnibus",
  },
  {
    key: "chartOfAccountPaymentsMadeOmnibusParentCode",
    defaultCode: "9110",
    category: "offBalanceSheet",
    group: "omnibus",
  },
  {
    key: "chartOfAccountInterestAddedToObligationsOmnibusParentCode",
    defaultCode: "9110",
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
    defaultCode: "9110",
    category: "offBalanceSheet",
    group: "summary",
  },
  {
    key: "chartOfAccountDisbursedDefaultedParentCode",
    defaultCode: "11.02.0203",
    category: "asset",
    group: "summary",
  },
  {
    key: "chartOfAccountInterestDefaultedParentCode",
    defaultCode: "11.02.0203",
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

const defaultFormData = CREDIT_CONFIG_FIELDS.reduce(
  (acc, field) => {
    acc[field.key] = field.defaultCode
    return acc
  },
  {} as CreditModuleConfigureInput,
)

const FIELD_GROUPS: Array<{
  key: CreditConfigField["group"]
  titleKey: "omnibus" | "summary"
}> = [
  { key: "omnibus", titleKey: "omnibus" },
  { key: "summary", titleKey: "summary" },
]

const emptyFormData = CREDIT_CONFIG_FIELDS.reduce(
  (acc, field) => {
    acc[field.key] = ""
    return acc
  },
  {} as CreditModuleConfigureInput,
)

type ChangeItem = {
  field: CreditConfigField
  from: string
  to: string
}

type AccountSetComboboxProps = {
  id?: string
  value: string
  options: AccountSetOption[]
  onChange: (value: string) => void
  disabled?: boolean
  placeholder: string
  searchPlaceholder: string
  emptyLabel: string
}

const AccountSetCombobox = ({
  id,
  value,
  options,
  onChange,
  disabled,
  placeholder,
  searchPlaceholder,
  emptyLabel,
}: AccountSetComboboxProps) => {
  const [open, setOpen] = useState(false)
  const selectedOption = options.find((option) => option.code === value)
  const selectedSecondary = selectedOption?.ref ?? selectedOption?.code
  const displayValue = selectedOption
    ? `${selectedOption.name} - ${selectedSecondary ?? ""}`.trim()
    : value
  const displayText = displayValue || placeholder
  const showTooltip = Boolean(displayValue)

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          id={id}
          variant="outline"
          role="combobox"
          aria-expanded={open}
          disabled={disabled}
          className="w-full justify-between font-normal"
        >
          {showTooltip ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="min-w-0 flex-1 truncate text-left">
                  {displayText}
                </span>
              </TooltipTrigger>
              <TooltipContent>{displayValue}</TooltipContent>
            </Tooltip>
          ) : (
            <span className="min-w-0 flex-1 truncate text-left text-muted-foreground">
              {displayText}
            </span>
          )}
          <ChevronsUpDownIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className="w-[calc(var(--radix-popover-trigger-width)*0.7)] min-w-[calc(var(--radix-popover-trigger-width)*0.5)] max-w-[calc(var(--radix-popover-trigger-width)*0.75)] p-0"
        align="start"
      >
        <Command>
          <CommandInput placeholder={searchPlaceholder} />
          <CommandList>
            <CommandEmpty>{emptyLabel}</CommandEmpty>
            <CommandGroup>
              {options.map((option) => {
                const optionSecondary = option.ref ?? option.code
                const optionLabel = optionSecondary
                  ? `${option.name} - ${optionSecondary}`
                  : option.name
                const keywords = [option.name, option.ref, option.code].filter(
                  (keyword): keyword is string => Boolean(keyword),
                )

                return (
                  <CommandItem
                    key={`${option.code}-${option.ref ?? "no-ref"}`}
                    value={option.code}
                    keywords={keywords}
                    onSelect={() => {
                      onChange(option.code)
                      setOpen(false)
                    }}
                  >
                    <CheckIcon
                      className={cn(
                        "mr-2 h-4 w-4",
                        value === option.code ? "opacity-100" : "opacity-0",
                      )}
                    />
                    <span className="truncate">{optionLabel}</span>
                  </CommandItem>
                )
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}

const buildFormDataFromConfig = (
  creditModuleConfig?: CreditModuleConfig,
): CreditModuleConfigureInput => {
  const updatedFormData = { ...emptyFormData }
  if (!creditModuleConfig) return updatedFormData

  CREDIT_CONFIG_FIELDS.forEach((field) => {
    const value = creditModuleConfig[field.key as keyof CreditModuleConfig]
    if (value) {
      updatedFormData[field.key] = value as string
    }
  })

  return updatedFormData
}

const buildChanges = (
  baseline: CreditModuleConfigureInput,
  current: CreditModuleConfigureInput,
): ChangeItem[] =>
  CREDIT_CONFIG_FIELDS.flatMap((field) => {
    const from = baseline[field.key] ?? ""
    const to = current[field.key] ?? ""
    if (from === to) return []
    return [{ field, from, to }]
  })

const formatOptionValue = (value: string, options: AccountSetOption[]) => {
  if (!value) return ""
  const match = options.find((option) => option.code === value)
  if (!match) return value
  const secondary = match.ref ?? match.code
  return `${match.name} - ${secondary}`.trim()
}

export const CreditConfigUpdateDialog: React.FC<CreditConfigUpdateDialogProps> = ({
  open,
  setOpen,
  creditModuleConfig,
  accountSetOptions = [],
}) => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [updateCreditConfig, { loading, error, reset }] =
    useCreditModuleConfigureMutation({
      refetchQueries: [CreditConfigDocument],
    })
  const [step, setStep] = useState<"edit" | "confirm">("edit")
  const [confirmationChanges, setConfirmationChanges] = useState<ChangeItem[]>([])
  const [baselineFormData, setBaselineFormData] =
    useState<CreditModuleConfigureInput>(emptyFormData)
  const [formData, setFormData] =
    useState<CreditModuleConfigureInput>(emptyFormData)
  const prevOpenRef = useRef(false)
  const accountSetOptionsByCategory = useMemo(() => {
    const grouped: Record<CreditAccountCategoryKey, AccountSetOption[]> = {
      offBalanceSheet: [],
      asset: [],
      liability: [],
      equity: [],
      revenue: [],
      costOfRevenue: [],
      expenses: [],
    }

    accountSetOptions.forEach((option) => {
      grouped[option.category].push(option)
    })

    return grouped
  }, [accountSetOptions])
  const editChanges = useMemo(
    () => buildChanges(baselineFormData, formData),
    [baselineFormData, formData],
  )
  const hasChanges = editChanges.length > 0

  const close = () => {
    reset()
    setOpen(false)
    setFormData({ ...emptyFormData })
    setBaselineFormData({ ...emptyFormData })
    setConfirmationChanges([])
    setStep("edit")
  }

  useEffect(() => {
    if (open && !prevOpenRef.current) {
      const updatedFormData = buildFormDataFromConfig(creditModuleConfig)
      setBaselineFormData(updatedFormData)
      setFormData(updatedFormData)
      setConfirmationChanges([])
      setStep("edit")
    }
    prevOpenRef.current = open
  }, [creditModuleConfig, open])

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    const changes = buildChanges(baselineFormData, formData)
    reset()
    setConfirmationChanges(changes)
    setStep("confirm")
  }

  const autoPopulate = () => {
    setFormData({ ...defaultFormData })
  }

  const handleDone = async () => {
    try {
      await updateCreditConfig({ variables: { input: formData } })
      toast.success(t("credit.updateSuccess"))
      close()
    } catch {
      // error is rendered inline in confirmation view
    }
  }

  const handleBack = () => {
    setStep("edit")
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) close()
      }}
    >
      <DialogContent className="max-h-[calc(100vh-2rem)] overflow-y-auto sm:max-w-4xl">
        <DialogHeader>
          <DialogTitle>
            {step === "confirm" ? t("credit.confirmationTitle") : t("credit.setTitle")}
          </DialogTitle>
        </DialogHeader>
        {step === "confirm" ? (
          <>
            <p className="text-sm text-muted-foreground">
              {t("credit.confirmationDescription")}
            </p>
            {confirmationChanges.length === 0 ? (
              <div className="mt-4 rounded-md border border-dashed border-border p-3 text-sm text-muted-foreground">
                {t("credit.confirmationNoChanges")}
              </div>
            ) : (
              <div className="mt-4 space-y-3">
                {confirmationChanges.map(({ field, from, to }) => {
                  const optionsForCategory = accountSetOptionsByCategory[field.category]
                  const previousLabel = formatOptionValue(from, optionsForCategory)
                  const updatedLabel = formatOptionValue(to, optionsForCategory)
                  const previousEmpty = !from
                  const updatedEmpty = !to

                  return (
                    <div
                      key={field.key}
                      className="space-y-2 rounded-lg border border-border p-3"
                    >
                      <div className="text-sm font-medium">
                        {t(`credit.${field.key}`)}
                      </div>
                      <div className="grid gap-3 sm:grid-cols-2">
                        <div className="space-y-1">
                          <div className="text-xs text-muted-foreground">
                            {t("credit.confirmationPrevious")}
                          </div>
                          <div
                            className={cn(
                              "min-h-[2rem] rounded-md border px-2 py-1 text-sm",
                              previousEmpty
                                ? "border-amber-400/70 text-amber-700"
                                : "border-border",
                            )}
                          >
                            {previousEmpty ? "\u00A0" : previousLabel}
                          </div>
                        </div>
                        <div className="space-y-1">
                          <div className="text-xs text-muted-foreground">
                            {t("credit.confirmationUpdated")}
                          </div>
                          <div
                            className={cn(
                              "min-h-[2rem] rounded-md border px-2 py-1 text-sm",
                              updatedEmpty
                                ? "border-amber-400/70 text-amber-700"
                                : "border-border",
                            )}
                          >
                            {updatedEmpty ? "\u00A0" : updatedLabel}
                          </div>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
            {error && <div className="text-destructive">{error.message}</div>}
            <DialogFooter className="mt-4">
              <Button variant="outline" type="button" onClick={handleBack}>
                {tCommon("back")}
              </Button>
              <Button loading={loading} type="button" onClick={handleDone}>
                {tCommon("save")}
              </Button>
            </DialogFooter>
          </>
        ) : (
          <form onSubmit={submit}>
            <div className="flex flex-col space-y-6 w-full">
              {FIELD_GROUPS.map((group) => {
                const fields = CREDIT_CONFIG_FIELDS.filter(
                  (field) => field.group === group.key,
                )

                return (
                  <div
                    key={group.key}
                    className="space-y-3 rounded-lg border border-border bg-muted/30 p-4"
                  >
                    <div className="text-sm font-semibold">
                      {t(`credit.groups.${group.titleKey}`)}
                    </div>
                    <div className="flex flex-col space-y-2 w-full">
                      {fields.map((field) => {
                        const optionsForCategory = accountSetOptionsByCategory[field.category]
                        const isDisabled = optionsForCategory.length === 0
                        const handleChange = (nextValue: string) => {
                          setFormData({ ...formData, [field.key]: nextValue })
                        }

                        return (
                          <div key={field.key}>
                            <div className="flex items-center justify-between gap-2">
                              <Label htmlFor={field.key}>
                                {t(`credit.${field.key}`)}
                              </Label>
                              <span className="text-xs text-muted-foreground">
                                {t(`accountCategories.${field.category}`)}
                              </span>
                            </div>
                            <AccountSetCombobox
                              id={field.key}
                              value={formData[field.key]}
                              options={optionsForCategory}
                              onChange={handleChange}
                              disabled={isDisabled}
                              placeholder={t("credit.accountSetSelectPlaceholder")}
                              searchPlaceholder={t("credit.accountSetSearchPlaceholder")}
                              emptyLabel={t("credit.accountSetEmpty")}
                            />
                          </div>
                        )
                      })}
                    </div>
                  </div>
                )
              })}
            </div>
            {error && <div className="text-destructive">{error.message}</div>}
            <DialogFooter className="mt-4">
              <Button
                variant="outline"
                type="button"
                onClick={autoPopulate}
                className="mr-auto"
              >
                {t("autoPopulate")}
              </Button>
              <Button variant="outline" type="button" onClick={close}>
                {tCommon("cancel")}
              </Button>
              <Button loading={loading} type="submit" disabled={!hasChanges || loading}>
                {tCommon("review")}
              </Button>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  )
}
