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
import { FormEvent, useEffect, useMemo, useState } from "react"
import { toast } from "sonner"

import {
  CREDIT_CONFIG_FIELDS,
  CREDIT_DEFAULT_FORM_DATA,
  CREDIT_EMPTY_FORM_DATA,
  CREDIT_FIELD_GROUPS,
  CreditAccountCategoryKey,
  buildCreditChanges,
  buildCreditFormDataFromConfig,
} from "./credit-config-fields"

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

type AccountSetOption = {
  accountSetId: string
  code: string
  name: string
  category: CreditAccountCategoryKey
}

type CreditConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  creditModuleConfig?: CreditModuleConfig
  accountSetOptions?: AccountSetOption[]
  accountSetOptionsError?: boolean
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

const getOptionLabel = (option: AccountSetOption) => {
  return option.code ? `${option.name} - ${option.code}` : option.name
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
  const displayValue = selectedOption ? getOptionLabel(selectedOption) : value
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
        className="w-[--radix-popover-trigger-width] p-0"
        align="start"
      >
        <Command>
          <CommandInput placeholder={searchPlaceholder} />
          <CommandList>
            <CommandEmpty>{emptyLabel}</CommandEmpty>
            <CommandGroup>
              {options.map((option) => {
                const optionLabel = getOptionLabel(option)
                const keywords = [option.name, option.code].filter(
                  (keyword): keyword is string => Boolean(keyword),
                )

                return (
                  <CommandItem
                    key={option.accountSetId}
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

const formatOptionValue = (value: string, options: AccountSetOption[]) => {
  if (!value) return ""
  const match = options.find((option) => option.code === value)
  if (!match) return value
  return getOptionLabel(match)
}

export const CreditConfigUpdateDialog: React.FC<CreditConfigUpdateDialogProps> = ({
  open,
  setOpen,
  creditModuleConfig,
  accountSetOptions = [],
  accountSetOptionsError = false,
}) => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [updateCreditConfig, { loading, error, reset }] =
    useCreditModuleConfigureMutation({
      refetchQueries: [CreditConfigDocument],
    })
  const [step, setStep] = useState<"edit" | "confirm">("edit")
  const [baselineFormData, setBaselineFormData] =
    useState<CreditModuleConfigureInput>({ ...CREDIT_EMPTY_FORM_DATA })
  const [formData, setFormData] =
    useState<CreditModuleConfigureInput>({ ...CREDIT_EMPTY_FORM_DATA })
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
  const changes = useMemo(
    () => buildCreditChanges(baselineFormData, formData),
    [baselineFormData, formData],
  )
  const hasChanges = changes.length > 0

  const close = () => {
    reset()
    setOpen(false)
    setFormData({ ...CREDIT_EMPTY_FORM_DATA })
    setBaselineFormData({ ...CREDIT_EMPTY_FORM_DATA })
    setStep("edit")
  }

  useEffect(() => {
    if (!open) return
    const updatedFormData = buildCreditFormDataFromConfig(creditModuleConfig)
    setBaselineFormData(updatedFormData)
    setFormData(updatedFormData)
    setStep("edit")
  }, [creditModuleConfig, open])

  const submit = (e: FormEvent) => {
    e.preventDefault()
    reset()
    setStep("confirm")
  }

  const autoPopulate = () => {
    setFormData({ ...CREDIT_DEFAULT_FORM_DATA })
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
            {changes.length > 0 && (
              <div className="mt-4 space-y-3">
                {changes.map(({ field, from, to }) => {
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
              {accountSetOptionsError && (
                <div className="text-sm text-destructive">
                  {tCommon("error")}
                </div>
              )}
              {CREDIT_FIELD_GROUPS.map((group) => {
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
