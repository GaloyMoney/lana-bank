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
  ProfitAndLossStatementConfigDocument,
  ProfitAndLossStatementModuleConfig,
  ProfitAndLossModuleConfigureInput,
  useProfitAndLossStatementConfigureMutation,
} from "@/lib/graphql/generated"

gql`
  mutation ProfitAndLossStatementConfigure($input: ProfitAndLossModuleConfigureInput!) {
    profitAndLossStatementConfigure(input: $input) {
      profitAndLossConfig {
        chartOfAccountsId
        chartOfAccountsRevenueCode
        chartOfAccountsCostOfRevenueCode
        chartOfAccountsExpensesCode
      }
    }
  }
`

type ProfitAndLossConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  profitAndLossConfig?: ProfitAndLossStatementModuleConfig
}

const initialFormData = {
  chartOfAccountsRevenueCode: "",
  chartOfAccountsCostOfRevenueCode: "",
  chartOfAccountsExpensesCode: "",
}

const pnlAccountCodes = {
  chartOfAccountsRevenueCode: "6",
  chartOfAccountsCostOfRevenueCode: "7",
  chartOfAccountsExpensesCode: "8",
}

export const ProfitAndLossConfigUpdateDialog: React.FC<
  ProfitAndLossConfigUpdateDialogProps
> = ({ open, setOpen, profitAndLossConfig }) => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [updateProfitAndLossConfig, { loading, error, reset }] =
    useProfitAndLossStatementConfigureMutation({
      refetchQueries: [ProfitAndLossStatementConfigDocument],
    })
  const [formData, setFormData] =
    useState<ProfitAndLossModuleConfigureInput>(initialFormData)

  const close = () => {
    reset()
    setOpen(false)
    setFormData(initialFormData)
  }

  useEffect(() => {
    if (
      profitAndLossConfig &&
      profitAndLossConfig.chartOfAccountsRevenueCode &&
      profitAndLossConfig.chartOfAccountsCostOfRevenueCode &&
      profitAndLossConfig.chartOfAccountsExpensesCode
    ) {
      setFormData({
        chartOfAccountsRevenueCode: profitAndLossConfig.chartOfAccountsRevenueCode,
        chartOfAccountsCostOfRevenueCode:
          profitAndLossConfig.chartOfAccountsCostOfRevenueCode,
        chartOfAccountsExpensesCode: profitAndLossConfig.chartOfAccountsExpensesCode,
      })
    }
  }, [profitAndLossConfig])

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    await updateProfitAndLossConfig({ variables: { input: formData } })
    close()
  }

  const autoPopulate = () => {
    setFormData(pnlAccountCodes)
  }

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("profitAndLoss.setTitle")}</DialogTitle>
        </DialogHeader>
        <form onSubmit={submit}>
          <div className="flex flex-col space-y-2 w-full">
            {Object.entries(formData).map(([key, value]) => (
              <div key={key}>
                <Label htmlFor={key}>{t(`profitAndLoss.${key}`)}</Label>
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
