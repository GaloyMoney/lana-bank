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
  useProspectConvertMutation,
  GetProspectBasicDetailsDocument,
} from "@/lib/graphql/generated"

type ConvertProspectDialogProps = {
  setOpenConvertDialog: (isOpen: boolean) => void
  openConvertDialog: boolean
  prospectId: string
}

export const ConvertProspectDialog: React.FC<ConvertProspectDialogProps> = ({
  setOpenConvertDialog,
  openConvertDialog,
  prospectId,
}) => {
  const t = useTranslations("Prospects.ProspectDetails.convertProspect")
  const commonT = useTranslations("Common")

  const [convertProspect, { loading, reset }] = useProspectConvertMutation({
    refetchQueries: [GetProspectBasicDetailsDocument],
  })
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      const result = await convertProspect({
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
      console.error("Error converting prospect:", error)
      setError(error instanceof Error ? error.message : commonT("error"))
    }
  }

  const handleCloseDialog = () => {
    setOpenConvertDialog(false)
    setError(null)
    reset()
  }

  return (
    <Dialog open={openConvertDialog} onOpenChange={handleCloseDialog}>
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
              disabled={loading}
              data-testid="confirm-convert-prospect-btn"
            >
              {t("buttons.convert")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default ConvertProspectDialog
