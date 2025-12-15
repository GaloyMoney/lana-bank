"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { useState, type ReactNode } from "react"
import { toast } from "sonner"

import { Dialog, DialogContent, DialogFooter } from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"

import {
  FiscalYear,
  useFiscalYearCloseMutation,
  FiscalYearsDocument,
} from "@/lib/graphql/generated"
import { useDialogSnapshot } from "@/hooks/use-dialog-snapshot"
import { useFiscalYearCloseConfirmation } from "@/hooks/use-fiscal-year-close-confirmation"
import { FiscalYearCloseDialogContent } from "@/components/fiscal-year-close-dialog-content"

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

  const fiscalYearSnapshot = useDialogSnapshot(fiscalYear, open)

  const confirmation = useFiscalYearCloseConfirmation(fiscalYearSnapshot)
  const confirmationLabel: ReactNode = confirmation.confirmationText
    ? t.rich("confirmationLabel", {
        text: confirmation.confirmationText,
        mono: (chunks: ReactNode) => (
          <span className="font-mono font-semibold text-foreground mx-1">{chunks}</span>
        ),
      })
    : null
  const [closeYearMutation, { loading }] = useFiscalYearCloseMutation({
    refetchQueries: [FiscalYearsDocument],
  })

  const resetState = () => {
    setError(null)
    confirmation.reset()
  }

  const handleCloseYear = async () => {
    setError(null)
    try {
      await closeYearMutation({
        variables: {
          input: {
            fiscalYearId: fiscalYearSnapshot.fiscalYearId,
          },
        },
      })
      toast.success(t("success"))
      resetState()
      onOpenChange(false)
    } catch (err) {
      setError(err instanceof Error ? err.message : tCommon("error"))
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        onOpenChange(isOpen)
        if (!isOpen) resetState()
      }}
    >
      <DialogContent className="sm:max-w-md">
        <FiscalYearCloseDialogContent
          title={t("title")}
          content={{
            description: t("description"),
            warning: t("warning"),
            closingLabel: t("closingYear"),
            closingValue: confirmation.displayText,
          }}
          confirmation={{
            label: confirmationLabel,
            expectedValue: confirmation.confirmationText,
            placeholder: t("placeholder"),
            value: confirmation.input,
            onChange: confirmation.setInput,
          }}
          state={{
            error,
            loading,
          }}
        />

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => {
              resetState()
              onOpenChange(false)
            }}
            disabled={loading}
          >
            {tCommon("cancel")}
          </Button>
          <Button
            onClick={handleCloseYear}
            loading={loading}
            disabled={!confirmation.isValid}
          >
            {t("confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
