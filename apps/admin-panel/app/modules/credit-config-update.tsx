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
        chartOfAccountFacilityOmnibusParentCode
        chartOfAccountCollateralOmnibusParentCode
        chartOfAccountFacilityParentCode
        chartOfAccountCollateralParentCode
        chartOfAccountInterestIncomeParentCode
        chartOfAccountFeeIncomeParentCode
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
      }
    }
  }
`

type CreditConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  creditModuleConfig?: CreditModuleConfig
}

const initialFormData: CreditModuleConfigureInput = {
  chartOfAccountFacilityOmnibusParentCode: "",
  chartOfAccountCollateralOmnibusParentCode: "",
  chartOfAccountFacilityParentCode: "",
  chartOfAccountCollateralParentCode: "",
  chartOfAccountInterestIncomeParentCode: "",
  chartOfAccountFeeIncomeParentCode: "",
  chartOfAccountShortTermIndividualDisbursedReceivableParentCode: "",
  chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode: "",
  chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode: "",
  chartOfAccountShortTermBankDisbursedReceivableParentCode: "",
  chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode: "",
  chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode: "",
  chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode: "",
  chartOfAccountLongTermIndividualDisbursedReceivableParentCode: "",
  chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode: "",
  chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode: "",
  chartOfAccountLongTermBankDisbursedReceivableParentCode: "",
  chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode: "",
  chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode: "",
  chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode: "",
  chartOfAccountShortTermIndividualInterestReceivableParentCode: "",
  chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode: "",
  chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode: "",
  chartOfAccountShortTermBankInterestReceivableParentCode: "",
  chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode: "",
  chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode: "",
  chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode: "",
  chartOfAccountLongTermIndividualInterestReceivableParentCode: "",
  chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode: "",
  chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode: "",
  chartOfAccountLongTermBankInterestReceivableParentCode: "",
  chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode: "",
  chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode: "",
  chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode: "",
}

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
  const [formData, setFormData] = useState<CreditModuleConfigureInput>(initialFormData)

  const close = () => {
    reset()
    setOpen(false)
    setFormData(initialFormData)
  }

  useEffect(() => {
    console.log(creditModuleConfig)
    if (creditModuleConfig) {
      const updatedFormData = { ...initialFormData }
      Object.keys(initialFormData).forEach((key) => {
        console.log(`key: ${key}`)
        if (creditModuleConfig[key as keyof CreditModuleConfig]) {
          console.log(`value: ${creditModuleConfig[key as keyof CreditModuleConfig]}`)
          updatedFormData[key as keyof CreditModuleConfigureInput] = creditModuleConfig[
            key as keyof CreditModuleConfig
          ] as string
        }
      })
      if (Object.values(updatedFormData).some((value) => value !== "")) {
        setFormData(updatedFormData)
      }
    }
  }, [creditModuleConfig])

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    await updateCreditConfig({ variables: { input: formData } })
    setOpen(false)
  }

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("credit.setTitle")}</DialogTitle>
        </DialogHeader>
        <form onSubmit={submit}>
          <div className="flex flex-col space-y-2 w-full">
            {Object.entries(formData).map(([key, value]) => (
              <div key={key}>
                <Label htmlFor={key}>{t(`credit.${key}`)}</Label>
                <Input
                  id={key}
                  value={value}
                  onChange={(e) => setFormData({ ...formData, [key]: e.target.value })}
                  required={true}
                />
              </div>
            ))}
          </div>
          {error && <div className="text-destructive">{error.message}</div>}
          <DialogFooter className="mt-4">
            <Button variant="outline" onClick={close}>
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
