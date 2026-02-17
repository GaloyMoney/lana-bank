"use client"

import { gql } from "@apollo/client"
import { Button } from "@lana/web/ui/button"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Label } from "@lana/web/ui/label"
import { cn } from "@lana/web/utils"
import { useTranslations } from "next-intl"
import { FormEvent, useEffect, useMemo, useState } from "react"
import { toast } from "sonner"

import {
  DEPOSIT_CONFIG_FIELDS,
  DEPOSIT_EMPTY_FORM_DATA,
  DEPOSIT_FIELD_GROUPS,
  DepositAccountCategoryKey,
  buildDepositChanges,
  buildDepositFormDataFromConfig,
} from "./deposit-config-fields"

import {
  DepositConfigDocument,
  DepositModuleConfig,
  DepositModuleConfigureInput,
  useDepositModuleConfigureMutation,
} from "@/lib/graphql/generated"
import {
  AccountSetCombobox,
  formatOptionValue,
  type AccountSetOptionBase,
} from "@/app/components/account-set-combobox"

gql`
  mutation DepositModuleConfigure($input: DepositModuleConfigureInput!) {
    depositModuleConfigure(input: $input) {
      depositConfig {
        chartOfAccountsId
        chartOfAccountsOmnibusParentCode
        chartOfAccountsIndividualDepositAccountsParentCode
        chartOfAccountsGovernmentEntityDepositAccountsParentCode
        chartOfAccountPrivateCompanyDepositAccountsParentCode
        chartOfAccountBankDepositAccountsParentCode
        chartOfAccountFinancialInstitutionDepositAccountsParentCode
        chartOfAccountNonDomiciledCompanyDepositAccountsParentCode
        chartOfAccountsFrozenIndividualDepositAccountsParentCode
        chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode
        chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode
        chartOfAccountFrozenBankDepositAccountsParentCode
        chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode
        chartOfAccountFrozenNonDomiciledCompanyDepositAccountsParentCode
      }
    }
  }
`

type DepositConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  depositModuleConfig?: DepositModuleConfig
  accountSetOptions?: AccountSetOption[]
  accountSetOptionsError?: boolean
}

type AccountSetOption = AccountSetOptionBase & {
  category: DepositAccountCategoryKey
}

export const DepositConfigUpdateDialog: React.FC<DepositConfigUpdateDialogProps> = ({
  open,
  setOpen,
  depositModuleConfig,
  accountSetOptions = [],
  accountSetOptionsError = false,
}) => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [updateDepositConfig, { loading, error, reset }] =
    useDepositModuleConfigureMutation({
      refetchQueries: [DepositConfigDocument],
    })
  const [step, setStep] = useState<"edit" | "confirm">("edit")
  const [baselineFormData, setBaselineFormData] =
    useState<DepositModuleConfigureInput>({ ...DEPOSIT_EMPTY_FORM_DATA })
  const [formData, setFormData] =
    useState<DepositModuleConfigureInput>({ ...DEPOSIT_EMPTY_FORM_DATA })
  const accountSetOptionsByCategory = useMemo(() => {
    const grouped: Record<DepositAccountCategoryKey, AccountSetOption[]> = {
      asset: [],
      liability: [],
    }

    accountSetOptions.forEach((option) => {
      grouped[option.category].push(option)
    })

    return grouped
  }, [accountSetOptions])
  const changes = useMemo(
    () => buildDepositChanges(baselineFormData, formData),
    [baselineFormData, formData],
  )
  const hasChanges = changes.length > 0

  const close = () => {
    reset()
    setOpen(false)
    setFormData({ ...DEPOSIT_EMPTY_FORM_DATA })
    setBaselineFormData({ ...DEPOSIT_EMPTY_FORM_DATA })
    setStep("edit")
  }

  useEffect(() => {
    if (!open) return
    const updatedFormData = buildDepositFormDataFromConfig(depositModuleConfig)
    setBaselineFormData(updatedFormData)
    setFormData(updatedFormData)
    setStep("edit")
  }, [depositModuleConfig, open])

  const submit = (e: FormEvent) => {
    e.preventDefault()
    reset()
    setStep("confirm")
  }

  const handleDone = async () => {
    try {
      await updateDepositConfig({ variables: { input: formData } })
      toast.success(t("deposit.updateSuccess"))
      close()
    } catch {
      // error is rendered inline in confirmation view
    }
  }

  const handleBack = () => {
    reset()
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
            {step === "confirm"
              ? t("deposit.confirmationTitle")
              : t("deposit.setTitle")}
          </DialogTitle>
        </DialogHeader>
        {step === "confirm" ? (
          <>
            <p className="text-sm text-muted-foreground">
              {t("deposit.confirmationDescription")}
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
                        {t(`deposit.${field.key}`)}
                      </div>
                      <div className="grid gap-3 sm:grid-cols-2">
                        <div className="space-y-1">
                          <div className="text-xs text-muted-foreground">
                            {t("deposit.confirmationPrevious")}
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
                            {t("deposit.confirmationUpdated")}
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
                <div className="text-sm text-destructive">{tCommon("error")}</div>
              )}
              {DEPOSIT_FIELD_GROUPS.map((group) => {
                const fields = DEPOSIT_CONFIG_FIELDS.filter(
                  (field) => field.group === group.key,
                )

                return (
                  <div
                    key={group.key}
                    className="space-y-3 rounded-lg border border-border bg-muted/30 p-4"
                  >
                    <div className="text-sm font-semibold">
                      {t(`deposit.groups.${group.titleKey}`)}
                    </div>
                    <div className="flex flex-col space-y-2 w-full">
                      {fields.map((field) => {
                        const optionsForCategory =
                          accountSetOptionsByCategory[field.category]
                        const isDisabled = optionsForCategory.length === 0
                        const handleChange = (nextValue: string) => {
                          setFormData({ ...formData, [field.key]: nextValue })
                        }

                        return (
                          <div key={field.key}>
                            <div className="flex items-center justify-between gap-2">
                              <Label htmlFor={field.key}>
                                {t(`deposit.${field.key}`)}
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
                              placeholder={t("deposit.accountSetSelectPlaceholder")}
                              searchPlaceholder={t("deposit.accountSetSearchPlaceholder")}
                              emptyLabel={t("deposit.accountSetEmpty")}
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
