"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { useState } from "react"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"

import { useModalNavigation } from "@/hooks/use-modal-navigation"
import { getUTCYear } from "@/utils/fiscal-year-dates"
import { DetailItem } from "@/components/details/item"
import { DetailsGroup } from "@/components/details/group"

import {
  FiscalYear,
  useFiscalYearOpenNextMutation,
  FiscalYearsDocument,
} from "@/lib/graphql/generated"

gql`
  mutation FiscalYearOpenNext($input: FiscalYearOpenNextInput!) {
    fiscalYearOpenNext(input: $input) {
      fiscalYear {
        ...FiscalYearDetailsPageFragment
      }
    }
  }
`

type OpenNextYearDialogProps = {
  fiscalYear: Pick<FiscalYear, "fiscalYearId" | "openedAsOf">
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function OpenNextYearDialog({
  fiscalYear,
  open,
  onOpenChange,
}: OpenNextYearDialogProps) {
  const t = useTranslations("FiscalYears.openNextYear")
  const tCommon = useTranslations("Common")
  const [error, setError] = useState<string | null>(null)

  const latestClosedYear = getUTCYear(fiscalYear.openedAsOf)
  const nextFiscalYear = latestClosedYear !== null ? latestClosedYear + 1 : null

  const { navigate } = useModalNavigation({
    closeModal: () => onOpenChange(false),
  })

  const [openNextYearMutation, { loading }] = useFiscalYearOpenNextMutation({
    refetchQueries: [FiscalYearsDocument],
  })

  const handleOpenNextYear = async () => {
    setError(null)
    try {
      const { data } = await openNextYearMutation({
        variables: {
          input: {
            fiscalYearId: fiscalYear.fiscalYearId,
          },
        },
      })
      toast.success(t("success"))
      const nextFiscalYearId = data?.fiscalYearOpenNext?.fiscalYear?.fiscalYearId
      if (nextFiscalYearId) {
        navigate(`/fiscal-years/${nextFiscalYearId}`)
      }
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
        <DialogDescription>{t("description")}</DialogDescription>
        {nextFiscalYear && (
          <DetailsGroup layout="horizontal">
            <DetailItem
              label={t("nextFiscalYearLabel")}
              value={nextFiscalYear}
              className="bg-muted/50 border rounded-lg p-2"
            />
          </DetailsGroup>
        )}
        {error && <p className="text-destructive text-sm">{error}</p>}
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {tCommon("cancel")}
          </Button>
          <Button onClick={handleOpenNextYear} loading={loading}>
            {t("confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
