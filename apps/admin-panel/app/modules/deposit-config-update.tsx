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
        chartOfAccountNonDomiciledCompanyDepositAccountsParentCode
      }
    }
  }
`

type DepositConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  depositModuleConfig?: DepositModuleConfig
}

const initialFormData = {
  chartOfAccountsOmnibusParentCode: "",
  chartOfAccountsIndividualDepositAccountsParentCode: "",
  chartOfAccountsGovernmentEntityDepositAccountsParentCode: "",
  chartOfAccountPrivateCompanyDepositAccountsParentCode: "",
  chartOfAccountBankDepositAccountsParentCode: "",
  chartOfAccountFinancialInstitutionDepositAccountsParentCode: "",
  chartOfAccountNonDomiciledIndividualDepositAccountsParentCode: "",
}

const depositModuleCodes = {
  chartOfAccountsOmnibusParentCode: "1110.01.0101",
  chartOfAccountsIndividualDepositAccountsParentCode: "2110.01.0401",
  chartOfAccountsGovernmentEntityDepositAccountsParentCode: "2110.01.0201",
  chartOfAccountPrivateCompanyDepositAccountsParentCode: "2110.01.0301",
  chartOfAccountBankDepositAccountsParentCode: "2110.01.0501",
  chartOfAccountFinancialInstitutionDepositAccountsParentCode: "2110.01.0601",
  chartOfAccountNonDomiciledIndividualDepositAccountsParentCode: "2110.01.0901",
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
      depositModuleConfig.chartOfAccountNonDomiciledCompanyDepositAccountsParentCode
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
          depositModuleConfig.chartOfAccountNonDomiciledCompanyDepositAccountsParentCode,
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
