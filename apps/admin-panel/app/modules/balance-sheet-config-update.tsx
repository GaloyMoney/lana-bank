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
  BalanceSheetConfigDocument,
  BalanceSheetModuleConfig,
  useBalanceSheetConfigureMutation,
} from "@/lib/graphql/generated"

gql`
  mutation BalanceSheetConfigure {
    balanceSheetConfigure {
      balanceSheetConfig {
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
  open: boolean,
  balanceSheetConfig?: BalanceSheetModuleConfig
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

  const close = () => {
    reset()
    setOpen(false)
  }


  const submit = async (e: FormEvent) => {
    e.preventDefault()
    await updateBalanceSheetConfig()
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
            {balanceSheetConfig &&
              Object.entries(balanceSheetConfig).map(
                ([key, value]) =>
                  key !== "__typename" && (
                    <div key={key}>
                      <Label htmlFor={key}>{t(`balanceSheet.${key}`)}</Label>
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
