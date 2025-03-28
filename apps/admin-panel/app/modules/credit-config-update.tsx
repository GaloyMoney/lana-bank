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
  chartOfAccountOverdueIndividualDisbursedReceivableParentCode: "",
  chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode: "",
  chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode: "",
  chartOfAccountOverdueBankDisbursedReceivableParentCode: "",
  chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode: "",
  chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode: "",
  chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode: "",
}

const creditModuleCodes = {
  chartOfAccountFacilityOmnibusParentCode: "9110.02.0201",
  chartOfAccountCollateralOmnibusParentCode: "9220.08.0201",
  chartOfAccountFacilityParentCode: "9110.02",
  chartOfAccountCollateralParentCode: "9220.08",
  chartOfAccountInterestIncomeParentCode: "6110.01.0100",
  chartOfAccountFeeIncomeParentCode: "6110.01.0300",
  chartOfAccountShortTermIndividualInterestReceivableParentCode: "1141.04.9901",
  chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode: "1141.02.9901",
  chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode: "1141.03.9901",
  chartOfAccountShortTermBankInterestReceivableParentCode: "1141.05.9901",
  chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode: "1141.06.9901",
  chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode:
    "1141.07.9901",
  chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode: "1141.08.9901",
  chartOfAccountLongTermIndividualInterestReceivableParentCode: "1142.04.9901",
  chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode: "1142.02.9901",
  chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode: "1142.03.9901",
  chartOfAccountLongTermBankInterestReceivableParentCode: "1142.05.9901",
  chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode: "1142.06.9901",
  chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode:
    "1142.07.9901",
  chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode: "1142.08.9901",
  chartOfAccountShortTermIndividualDisbursedReceivableParentCode: "1141.04.0101",
  chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode: "1141.02.0101",
  chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode: "1141.03.0101",
  chartOfAccountShortTermBankDisbursedReceivableParentCode: "1141.05.0401",
  chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode:
    "1141.06.0101",
  chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode:
    "1141.07.0101",
  chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode: "1141.08.0101",
  chartOfAccountLongTermIndividualDisbursedReceivableParentCode: "1142.04.0101",
  chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode: "1142.02.0101",
  chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode: "1142.03.0101",
  chartOfAccountLongTermBankDisbursedReceivableParentCode: "1142.05.0401",
  chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode: "1142.06.0101",
  chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode:
    "1142.07.0101",
  chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode: "1142.08.0101",
  chartOfAccountOverdueIndividualDisbursedReceivableParentCode: "1148.04.0101",
  chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode: "1148.02.0101",
  chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode: "1148.03.0101",
  chartOfAccountOverdueBankDisbursedReceivableParentCode: "1148.05.0401",
  chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode: "1148.06.0101",
  chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode:
    "1148.07.0101",
  chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode: "1148.08.0101",
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
    if (creditModuleConfig) {
      const updatedFormData = { ...initialFormData }
      Object.keys(initialFormData).forEach((key) => {
        if (creditModuleConfig[key as keyof CreditModuleConfig]) {
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

  const autoPopulate = () => {
    setFormData(creditModuleCodes)
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
