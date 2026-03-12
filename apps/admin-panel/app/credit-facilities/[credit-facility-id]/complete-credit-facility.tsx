"use client"

import React, { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
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

import {
  useCreditFacilityCompleteMutation,
  GetCreditFacilityLayoutDetailsDocument,
} from "@/lib/graphql/generated"

gql`
  mutation CreditFacilityComplete($input: CreditFacilityCompleteInput!) {
    creditFacilityComplete(input: $input) {
      creditFacility {
        id
        status
      }
    }
  }
`

type CompleteCreditFacilityDialogProps = {
  setOpenCompleteDialog: (isOpen: boolean) => void
  openCompleteDialog: boolean
  creditFacilityId: string
}

export const CompleteCreditFacilityDialog: React.FC<
  CompleteCreditFacilityDialogProps
> = ({ setOpenCompleteDialog, openCompleteDialog, creditFacilityId }) => {
  const t = useTranslations(
    "CreditFacilities.CreditFacilityDetails.completeCreditFacility",
  )
  const commonT = useTranslations("Common")

  const [completeCreditFacility, { loading, reset }] =
    useCreditFacilityCompleteMutation({
      refetchQueries: [GetCreditFacilityLayoutDetailsDocument],
    })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await completeCreditFacility({
        variables: {
          input: { creditFacilityId },
        },
      })

      if (result.data) {
        toast.success(t("success"))
        handleCloseDialog()
      }
    } catch (error) {
      console.error("Error completing credit facility:", error)
      setError(error instanceof Error ? error.message : commonT("error"))
    }
  }

  const handleCloseDialog = () => {
    setOpenCompleteDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openCompleteDialog} onOpenChange={handleCloseDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              variant="destructive"
              disabled={loading}
              data-testid="complete-credit-facility-dialog-button"
            >
              {t("buttons.complete")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default CompleteCreditFacilityDialog
