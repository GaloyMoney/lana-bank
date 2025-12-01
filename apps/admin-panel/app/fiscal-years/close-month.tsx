"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { useState } from "react"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"

import {
  FiscalYear,
  useFiscalYearCloseMonthMutation,
  FiscalYearsDocument,
} from "@/lib/graphql/generated"

gql`
  mutation FiscalYearCloseMonth($input: FiscalYearCloseMonthInput!) {
    fiscalYearCloseMonth(input: $input) {
      fiscalYear {
        ...FiscalYearDetailsPageFragment
      }
    }
  }
`

interface CloseMonthDialogProps {
  fiscalYear: FiscalYear
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CloseMonthDialog({
  fiscalYear,
  open,
  onOpenChange,
}: CloseMonthDialogProps) {
  const t = useTranslations("FiscalYears.closeMonth")
  const tCommon = useTranslations("Common")
  const [error, setError] = useState<string | null>(null)

  const [closeMonthMutation, { loading }] = useFiscalYearCloseMonthMutation({
    refetchQueries: [FiscalYearsDocument],
  })

  const handleCloseMonth = async () => {
    setError(null)
    try {
      await closeMonthMutation({
        variables: {
          input: {
            fiscalYearId: fiscalYear.fiscalYearId,
          },
        },
      })
      toast.success(t("success"))
      onOpenChange(false)
    } catch (err) {
      setError(err instanceof Error ? err.message : tCommon("error"))
    }
  }

  const resetState = () => {
    setError(null)
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        onOpenChange(isOpen)
        if (!isOpen) resetState()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
        </DialogHeader>

        <div className="bg-destructive/10 text-destructive text-sm p-3 rounded-md">
          {t("description")}
        </div>

        {error && <p className="text-destructive text-sm">{error}</p>}

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("cancel")}
          </Button>
          <Button onClick={handleCloseMonth} loading={loading}>
            {t("confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
