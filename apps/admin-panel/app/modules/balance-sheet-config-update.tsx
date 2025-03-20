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
  BalanceSheetConfigDocument,
  BalanceSheetModuleConfig,
  BalanceSheetModuleConfigureInput,
  useBalanceSheetConfigureMutation,
} from "@/lib/graphql/generated"

gql`
  mutation BalanceSheetConfigure($input: BalanceSheetModuleConfigureInput!) {
    balanceSheetConfigure(input: $input) {
      balanceSheetConfig {
        chartOfAccountsId
        chartOfAccountsAssetsCode
        chartOfAccountsLiabilitiesCode
        chartOfAccountsEquityCode
        chartOfAccountsRevenueCode
        chartOfAccountsCostOfRevenueCode
        chartOfAccountsExpensesCode
      }
    }
  }
`

type BalanceSheetConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  balanceSheetConfig?: BalanceSheetModuleConfig
}

const initialFormData = {
  chartOfAccountsAssetsCode: "",
  chartOfAccountsLiabilitiesCode: "",
  chartOfAccountsEquityCode: "",
  chartOfAccountsRevenueCode: "",
  chartOfAccountsCostOfRevenueCode: "",
  chartOfAccountsExpensesCode: "",
}

export const BalanceSheetConfigUpdateDialog: React.FC<
  BalanceSheetConfigUpdateDialogProps
> = ({ open, setOpen, balanceSheetConfig }) => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [updateBalanceSheetConfig, { loading, error, reset }] =
    useBalanceSheetConfigureMutation({
      refetchQueries: [BalanceSheetConfigDocument],
    })
  const [formData, setFormData] =
    useState<BalanceSheetModuleConfigureInput>(initialFormData)

  const close = () => {
    reset()
    setOpen(false)
    setFormData(initialFormData)
  }

  useEffect(() => {
    if (
      balanceSheetConfig &&
      balanceSheetConfig.chartOfAccountsAssetsCode &&
      balanceSheetConfig.chartOfAccountsLiabilitiesCode &&
      balanceSheetConfig.chartOfAccountsEquityCode &&
      balanceSheetConfig.chartOfAccountsRevenueCode &&
      balanceSheetConfig.chartOfAccountsCostOfRevenueCode &&
      balanceSheetConfig.chartOfAccountsExpensesCode
    ) {
      setFormData({
        chartOfAccountsAssetsCode: balanceSheetConfig.chartOfAccountsAssetsCode,
        chartOfAccountsLiabilitiesCode: balanceSheetConfig.chartOfAccountsLiabilitiesCode,
        chartOfAccountsEquityCode: balanceSheetConfig.chartOfAccountsEquityCode,
        chartOfAccountsRevenueCode: balanceSheetConfig.chartOfAccountsRevenueCode,
        chartOfAccountsCostOfRevenueCode:
          balanceSheetConfig.chartOfAccountsCostOfRevenueCode,
        chartOfAccountsExpensesCode: balanceSheetConfig.chartOfAccountsExpensesCode,
      })
    }
  }, [balanceSheetConfig])

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    await updateBalanceSheetConfig({ variables: { input: formData } })
    close()
  }

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("balanceSheet.setTitle")}</DialogTitle>
        </DialogHeader>
        <form onSubmit={submit}>
          <div className="flex flex-col space-y-2 w-full">
            {Object.entries(formData).map(([key, value]) => (
              <div key={key}>
                <Label htmlFor={key}>{t(`balanceSheet.${key}`)}</Label>
                <Input
                  id={key}
                  value={value}
                  required
                  onChange={(e) => setFormData({ ...formData, [key]: e.target.value })}
                />
              </div>
            ))}
            {error && <div className="text-destructive">{error.message}</div>}
          </div>
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
