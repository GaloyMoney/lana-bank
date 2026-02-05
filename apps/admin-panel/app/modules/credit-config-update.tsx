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
import { Input } from "@lana/web/ui/input"
import { useTranslations } from "next-intl"
import { FormEvent, useEffect, useState } from "react"

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

type CreditConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  creditModuleConfig?: CreditModuleConfig
}

type CreditAccountCategoryKey =
  | "offBalanceSheet"
  | "asset"
  | "liability"
  | "equity"
  | "revenue"
  | "costOfRevenue"
  | "expenses"

type CreditConfigField = {
  key: keyof CreditModuleConfigureInput
  defaultCode: string
  category: CreditAccountCategoryKey
}

const CREDIT_CONFIG_FIELDS: CreditConfigField[] = [
  {
    key: "chartOfAccountFacilityOmnibusParentCode",
    defaultCode: "9110.02.0201",
    category: "offBalanceSheet",
  },
  {
    key: "chartOfAccountCollateralOmnibusParentCode",
    defaultCode: "9220.08.0201",
    category: "offBalanceSheet",
  },
  {
    key: "chartOfAccountLiquidationProceedsOmnibusParentCode",
    defaultCode: "9170.00.0001",
    category: "revenue",
  },
  {
    key: "chartOfAccountPaymentsMadeOmnibusParentCode",
    defaultCode: "9110",
    category: "offBalanceSheet",
  },
  {
    key: "chartOfAccountInterestAddedToObligationsOmnibusParentCode",
    defaultCode: "9110",
    category: "offBalanceSheet",
  },
  {
    key: "chartOfAccountFacilityParentCode",
    defaultCode: "9110.02.0201",
    category: "offBalanceSheet",
  },
  {
    key: "chartOfAccountCollateralParentCode",
    defaultCode: "9220.08.0201",
    category: "offBalanceSheet",
  },
  {
    key: "chartOfAccountCollateralInLiquidationParentCode",
    defaultCode: "9220.08.0201",
    category: "offBalanceSheet",
  },
  {
    key: "chartOfAccountInterestIncomeParentCode",
    defaultCode: "6110.01.0100",
    category: "revenue",
  },
  {
    key: "chartOfAccountFeeIncomeParentCode",
    defaultCode: "6110.01.0300",
    category: "revenue",
  },
  {
    key: "chartOfAccountPaymentHoldingParentCode",
    defaultCode: "1141.99.0201",
    category: "asset",
  },
  {
    key: "chartOfAccountUncoveredOutstandingParentCode",
    defaultCode: "9110",
    category: "offBalanceSheet",
  },
  {
    key: "chartOfAccountShortTermIndividualDisbursedReceivableParentCode",
    defaultCode: "1141.04.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode",
    defaultCode: "1141.02.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode",
    defaultCode: "1141.03.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermBankDisbursedReceivableParentCode",
    defaultCode: "1141.05.0401",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode",
    defaultCode: "1141.06.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode",
    defaultCode: "1141.07.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode",
    defaultCode: "1141.08.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermIndividualDisbursedReceivableParentCode",
    defaultCode: "1142.04.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode",
    defaultCode: "1142.02.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode",
    defaultCode: "1142.03.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermBankDisbursedReceivableParentCode",
    defaultCode: "1142.05.0401",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode",
    defaultCode: "1142.06.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode",
    defaultCode: "1142.07.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode",
    defaultCode: "1142.08.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermIndividualInterestReceivableParentCode",
    defaultCode: "1141.04.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode",
    defaultCode: "1141.02.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode",
    defaultCode: "1141.03.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermBankInterestReceivableParentCode",
    defaultCode: "1141.05.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode",
    defaultCode: "1141.06.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode",
    defaultCode: "1141.07.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode",
    defaultCode: "1141.08.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermIndividualInterestReceivableParentCode",
    defaultCode: "1142.04.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode",
    defaultCode: "1142.02.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode",
    defaultCode: "1142.03.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermBankInterestReceivableParentCode",
    defaultCode: "1142.05.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode",
    defaultCode: "1142.06.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode",
    defaultCode: "1142.07.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode",
    defaultCode: "1142.08.9901",
    category: "asset",
  },
  {
    key: "chartOfAccountOverdueIndividualDisbursedReceivableParentCode",
    defaultCode: "1148.04.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode",
    defaultCode: "1148.02.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode",
    defaultCode: "1148.03.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountOverdueBankDisbursedReceivableParentCode",
    defaultCode: "1148.05.0401",
    category: "asset",
  },
  {
    key: "chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode",
    defaultCode: "1148.06.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode",
    defaultCode: "1148.07.0101",
    category: "asset",
  },
  {
    key: "chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode",
    defaultCode: "1148.08.0101",
    category: "asset",
  },
]

const defaultFormData = CREDIT_CONFIG_FIELDS.reduce(
  (acc, field) => {
    acc[field.key] = field.defaultCode
    return acc
  },
  {} as CreditModuleConfigureInput,
)

export const CreditConfigUpdateDialog: React.FC<CreditConfigUpdateDialogProps> = ({
  open,
  setOpen,
  creditModuleConfig,
}) => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [updateCreditConfig, { loading, error, reset }] =
    useCreditModuleConfigureMutation({
      refetchQueries: [CreditConfigDocument],
    })
  const [formData, setFormData] =
    useState<CreditModuleConfigureInput>(defaultFormData)

  const close = () => {
    reset()
    setOpen(false)
    setFormData({ ...defaultFormData })
  }

  useEffect(() => {
    if (!open) return
    if (!creditModuleConfig) {
      setFormData({ ...defaultFormData })
      return
    }

    const updatedFormData = { ...defaultFormData }
    CREDIT_CONFIG_FIELDS.forEach((field) => {
      const value = creditModuleConfig[field.key as keyof CreditModuleConfig]
      if (value) {
        updatedFormData[field.key] = value as string
      }
    })
    setFormData(updatedFormData)
  }, [creditModuleConfig, open])

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    await updateCreditConfig({ variables: { input: formData } })
    setOpen(false)
  }

  const autoPopulate = () => {
    setFormData({ ...defaultFormData })
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) close()
      }}
    >
      <DialogContent className="max-h-[calc(100vh-2rem)] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t("credit.setTitle")}</DialogTitle>
        </DialogHeader>
        <form onSubmit={submit}>
          <div className="flex flex-col space-y-2 w-full">
            {CREDIT_CONFIG_FIELDS.map((field) => (
              <div key={field.key}>
                <div className="flex items-center justify-between gap-2">
                  <Label htmlFor={field.key}>{t(`credit.${field.key}`)}</Label>
                  <span className="text-xs text-muted-foreground">
                    {t(`accountCategories.${field.category}`)}
                  </span>
                </div>
                <Input
                  id={field.key}
                  value={formData[field.key]}
                  onChange={(e) =>
                    setFormData({ ...formData, [field.key]: e.target.value })
                  }
                  required={true}
                />
              </div>
            ))}
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
            <Button loading={loading} type="submit">
              {tCommon("save")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
