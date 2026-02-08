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
  DepositConfigDocument,
  DepositModuleConfig,
  DepositModuleConfigureInput,
  useDepositModuleConfigureMutation,
} from "@/lib/graphql/generated"

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
        chartOfAccountNonDomiciledIndividualDepositAccountsParentCode
        chartOfAccountForeignAgencyOrSubsidiaryDepositAccountsParentCode
        chartOfAccountsFrozenIndividualDepositAccountsParentCode
        chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode
        chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode
        chartOfAccountFrozenBankDepositAccountsParentCode
        chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode
        chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode
        chartOfAccountFrozenForeignAgencyOrSubsidiaryDepositAccountsParentCode
      }
    }
  }
`

type DepositConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  depositModuleConfig?: DepositModuleConfig
}

const initialFormData: DepositModuleConfigureInput = {
  chartOfAccountsOmnibusParentCode: "",
  chartOfAccountsIndividualDepositAccountsParentCode: "",
  chartOfAccountsGovernmentEntityDepositAccountsParentCode: "",
  chartOfAccountPrivateCompanyDepositAccountsParentCode: "",
  chartOfAccountBankDepositAccountsParentCode: "",
  chartOfAccountFinancialInstitutionDepositAccountsParentCode: "",
  chartOfAccountNonDomiciledIndividualDepositAccountsParentCode: "",
  chartOfAccountForeignAgencyOrSubsidiaryDepositAccountsParentCode: "",
  chartOfAccountsFrozenIndividualDepositAccountsParentCode: "",
  chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode: "",
  chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode: "",
  chartOfAccountFrozenBankDepositAccountsParentCode: "",
  chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode: "",
  chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode: "",
  chartOfAccountFrozenForeignAgencyOrSubsidiaryDepositAccountsParentCode: "",
}

const depositModuleCodes = {
  chartOfAccountsOmnibusParentCode: "1110.01.0101",
  chartOfAccountsIndividualDepositAccountsParentCode: "2110.01.0401",
  chartOfAccountsGovernmentEntityDepositAccountsParentCode: "2110.01.0201",
  chartOfAccountPrivateCompanyDepositAccountsParentCode: "2110.01.0301",
  chartOfAccountBankDepositAccountsParentCode: "2110.01.0501",
  chartOfAccountFinancialInstitutionDepositAccountsParentCode: "2110.01.0601",
  chartOfAccountNonDomiciledIndividualDepositAccountsParentCode: "2110.01.0901",
  chartOfAccountForeignAgencyOrSubsidiaryDepositAccountsParentCode: "2110.01.0901",
  chartOfAccountsFrozenIndividualDepositAccountsParentCode: "2114.03.0401",
  chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode: "2114.03.0201",
  chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode: "2114.03.0301",
  chartOfAccountFrozenBankDepositAccountsParentCode: "2114.03.0501",
  chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode: "2114.03.0601",
  chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode: "2114.03.0701",
  chartOfAccountFrozenForeignAgencyOrSubsidiaryDepositAccountsParentCode: "2114.03.0701",
}

export const DepositConfigUpdateDialog: React.FC<DepositConfigUpdateDialogProps> = ({
  open,
  setOpen,
  depositModuleConfig,
}) => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [updateDepositConfig, { loading, error, reset }] =
    useDepositModuleConfigureMutation({
      refetchQueries: [DepositConfigDocument],
    })
  const [formData, setFormData] = useState<DepositModuleConfigureInput>(initialFormData)

  const close = () => {
    reset()
    setOpen(false)
    setFormData(initialFormData)
  }

  useEffect(() => {
    if (
      depositModuleConfig &&
      depositModuleConfig.chartOfAccountsOmnibusParentCode &&
      depositModuleConfig.chartOfAccountsIndividualDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountsGovernmentEntityDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountPrivateCompanyDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountBankDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountFinancialInstitutionDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountNonDomiciledIndividualDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountForeignAgencyOrSubsidiaryDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountsFrozenIndividualDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountFrozenBankDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode &&
      depositModuleConfig.chartOfAccountFrozenForeignAgencyOrSubsidiaryDepositAccountsParentCode
    ) {
      setFormData({
        chartOfAccountsOmnibusParentCode:
          depositModuleConfig.chartOfAccountsOmnibusParentCode,
        chartOfAccountsIndividualDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountsIndividualDepositAccountsParentCode,
        chartOfAccountsGovernmentEntityDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountsGovernmentEntityDepositAccountsParentCode,
        chartOfAccountPrivateCompanyDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountPrivateCompanyDepositAccountsParentCode,
        chartOfAccountBankDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountBankDepositAccountsParentCode,
        chartOfAccountFinancialInstitutionDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountFinancialInstitutionDepositAccountsParentCode,
        chartOfAccountNonDomiciledIndividualDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountNonDomiciledIndividualDepositAccountsParentCode,
        chartOfAccountForeignAgencyOrSubsidiaryDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountForeignAgencyOrSubsidiaryDepositAccountsParentCode,
        chartOfAccountsFrozenIndividualDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountsFrozenIndividualDepositAccountsParentCode,
        chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode,
        chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode,
        chartOfAccountFrozenBankDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountFrozenBankDepositAccountsParentCode,
        chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode,
        chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode,
        chartOfAccountFrozenForeignAgencyOrSubsidiaryDepositAccountsParentCode:
          depositModuleConfig.chartOfAccountFrozenForeignAgencyOrSubsidiaryDepositAccountsParentCode,
      })
    }
  }, [depositModuleConfig])

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    await updateDepositConfig({ variables: { input: formData } })
    close()
  }

  const autoPopulate = () => {
    setFormData(depositModuleCodes)
  }

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("deposit.setTitle")}</DialogTitle>
        </DialogHeader>
        <form onSubmit={submit}>
          <div className="flex flex-col space-y-2 w-full">
            {Object.entries(formData).map(([key, value]) => (
              <div key={key}>
                <Label htmlFor={key}>{t(`deposit.${key}`)}</Label>
                <Input
                  id={key}
                  value={value}
                  onChange={(e) => setFormData({ ...formData, [key]: e.target.value })}
                  required={true}
                />
              </div>
            ))}
            {error && <div className="text-destructive">{error.message}</div>}
          </div>
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
