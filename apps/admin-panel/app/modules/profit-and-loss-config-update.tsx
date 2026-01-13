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
import { FormEvent } from "react"

import {
  ProfitAndLossStatementConfigDocument,
  ProfitAndLossStatementModuleConfig,
  useProfitAndLossStatementConfigureMutation,
} from "@/lib/graphql/generated"

gql`
  mutation ProfitAndLossStatementConfigure {
    profitAndLossStatementConfigure {
      profitAndLossConfig {
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

export const ProfitAndLossConfigUpdateDialog: React.FC<
  ProfitAndLossConfigUpdateDialogProps
> = ({ open, setOpen, profitAndLossConfig }) => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [updateProfitAndLossConfig, { loading, error, reset }] =
    useProfitAndLossStatementConfigureMutation({
      refetchQueries: [ProfitAndLossStatementConfigDocument],
    })

  const close = () => {
    reset()
    setOpen(false)
  }

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    await updateProfitAndLossConfig()
    close()
  }

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("profitAndLoss.setTitle")}</DialogTitle>
        </DialogHeader>
        <form onSubmit={submit}>
        <div className="flex flex-col space-y-2 w-full">
            {profitAndLossConfig &&
              Object.entries(profitAndLossConfig).map(
                ([key, value]) =>
                  key !== "__typename" && (
                    <div key={key}>
                      <Label htmlFor={key}>{t(`profitAndLoss.${key}`)}</Label>
                      <Input id={key} value={value || ""} disabled />
                    </div>
                  ),
              )}
            {error && <div className="text-destructive">{error.message}</div>}
          </div>
          <DialogFooter className="mt-4">
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
