"use client"

import React, { useState } from "react"
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
  useProspectCloseMutation,
  GetProspectBasicDetailsDocument,
} from "@/lib/graphql/generated"

type CloseProspectDialogProps = {
  setOpenCloseDialog: (isOpen: boolean) => void
  openCloseDialog: boolean
  prospectId: string
}

export const CloseProspectDialog: React.FC<CloseProspectDialogProps> = ({
  setOpenCloseDialog,
  openCloseDialog,
  prospectId,
}) => {
  const t = useTranslations("Prospects.ProspectDetails.closeProspect")
  const commonT = useTranslations("Common")

  const [closeProspect, { loading, reset }] = useProspectCloseMutation({
    refetchQueries: [GetProspectBasicDetailsDocument],
  })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await closeProspect({
        variables: {
          input: {
            prospectId,
          },
        },
      })

      if (result.data) {
        toast.success(t("success"))
        handleCloseDialog()
      }
    } catch (error) {
      console.error("Error closing prospect:", error)
      setError(error instanceof Error ? error.message : commonT("error"))
    }
  }

  const handleCloseDialog = () => {
    setOpenCloseDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openCloseDialog} onOpenChange={handleCloseDialog}>
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
              data-testid="confirm-close-prospect-btn"
            >
              {t("buttons.close")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default CloseProspectDialog
