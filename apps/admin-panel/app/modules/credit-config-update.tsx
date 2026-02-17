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
  CREDIT_CONFIG_FIELDS,
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
import {
  AccountSetCombobox,
  formatOptionValue,
  type AccountSetOptionBase,
} from "@/app/components/account-set-combobox"

gql`
  mutation CreditModuleConfigure($input: CreditModuleConfigureInput!) {
    creditModuleConfigure(input: $input) {
      creditConfig {
        chartOfAccountsId
        chartOfAccountFacilityOmnibusParentCode
        chartOfAccountCollateralOmnibusParentCode
        chartOfAccountLiquidationProceedsOmnibusParentCode
        chartOfAccountPaymentsMadeOmnibusParentCode
        chartOfAccountInterestAddedToObligationsOmnibusParentCode
        chartOfAccountUncoveredOutstandingParentCode
        chartOfAccountFacilityParentCode
        chartOfAccountCollateralParentCode
        chartOfAccountCollateralInLiquidationParentCode
        chartOfAccountLiquidatedCollateralParentCode
        chartOfAccountProceedsFromLiquidationParentCode
        chartOfAccountInterestIncomeParentCode
        chartOfAccountFeeIncomeParentCode
        chartOfAccountPaymentHoldingParentCode
        chartOfAccountDisbursedDefaultedParentCode
        chartOfAccountInterestDefaultedParentCode
        chartOfAccountShortTermIndividualDisbursedReceivableParentCode
        chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode
        chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode
        chartOfAccountShortTermBankDisbursedReceivableParentCode
        chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode
        chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode
        chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode
        chartOfAccountLongTermIndividualDisbursedReceivableParentCode
        chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode
        chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode
        chartOfAccountLongTermBankDisbursedReceivableParentCode
        chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode
        chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode
        chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode
        chartOfAccountShortTermIndividualInterestReceivableParentCode
        chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode
        chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode
        chartOfAccountShortTermBankInterestReceivableParentCode
        chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode
        chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode
        chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode
        chartOfAccountLongTermIndividualInterestReceivableParentCode
        chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode
        chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode
        chartOfAccountLongTermBankInterestReceivableParentCode
        chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode
        chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode
        chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode
        chartOfAccountOverdueIndividualDisbursedReceivableParentCode
        chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode
        chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode
        chartOfAccountOverdueBankDisbursedReceivableParentCode
        chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode
        chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode
        chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode
      }
    }
  }
`

type AccountSetOption = AccountSetOptionBase & {
  category: CreditAccountCategoryKey
}

type CreditConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  creditModuleConfig?: CreditModuleConfig
  accountSetOptions?: AccountSetOption[]
  accountSetOptionsError?: boolean
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
