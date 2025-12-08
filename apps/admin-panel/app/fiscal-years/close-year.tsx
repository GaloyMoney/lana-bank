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
  useFiscalYearCloseMutation,
  FiscalYearsDocument,
} from "@/lib/graphql/generated"

gql`
  mutation FiscalYearClose($input: FiscalYearCloseInput!) {
    fiscalYearClose(input: $input) {
      fiscalYear {
        ...FiscalYearDetailsPageFragment
      }
    }
  }
`

type CloseYearDialogProps = {
  fiscalYear: FiscalYear
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CloseYearDialog({
  fiscalYear,
  open,
  onOpenChange,
}: CloseYearDialogProps) {
  const t = useTranslations("FiscalYears.closeYear")
  const tCommon = useTranslations("Common")
  const [error, setError] = useState<string | null>(null)

  const [closeYearMutation, { loading }] = useFiscalYearCloseMutation({
    refetchQueries: [FiscalYearsDocument],
  })

  const handleCloseYear = async () => {
    setError(null)
    try {
      await closeYearMutation({
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
            {tCommon("cancel")}
          </Button>
          <Button variant="destructive" onClick={handleCloseYear} loading={loading}>
            {t("confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
