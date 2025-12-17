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
import { Label } from "@lana/web/ui/label"
import { Calendar } from "@lana/web/ui/calendar"
import { Popover, PopoverContent, PopoverTrigger } from "@lana/web/ui/popover"
import { CalendarIcon } from "lucide-react"

import { formatUTCDateOnly } from "@lana/web/utils"

import { useFiscalYearInitMutation, FiscalYearsDocument } from "@/lib/graphql/generated"

gql`
  mutation FiscalYearInit($input: FiscalYearInitInput!) {
    fiscalYearInit(input: $input) {
      fiscalYear {
        ...FiscalYearFields
      }
    }
  }
`

interface InitFiscalYearDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function InitFiscalYearDialog({ open, onOpenChange }: InitFiscalYearDialogProps) {
  const t = useTranslations("FiscalYears.init")
  const tCommon = useTranslations("Common")
  const [openedAsOf, setOpenedAsOf] = useState<Date | undefined>(undefined)
  const [error, setError] = useState<string | null>(null)

  const [initMutation, { loading }] = useFiscalYearInitMutation({
    refetchQueries: [FiscalYearsDocument],
  })

  const handleInit = async () => {
    setError(null)

    if (!openedAsOf) {
      setError(t("validation.dateRequired"))
      return
    }

    try {
      await initMutation({
        variables: {
          input: {
            openedAsOf: openedAsOf.toISOString().split("T")[0],
          },
        },
      })
      toast.success(t("success"))
      onOpenChange(false)
      resetState()
    } catch (err) {
      setError(err instanceof Error ? err.message : tCommon("error"))
    }
  }

  const resetState = () => {
    setOpenedAsOf(undefined)
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
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <div className="bg-destructive/10 text-destructive text-sm p-3 rounded-md">
          {t("warning")}
        </div>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="openedAsOf">{t("fields.openedAsOf")}</Label>
            <Popover>
              <PopoverTrigger asChild>
                <Button
                  variant="outline"
                  className="w-full justify-start text-left font-normal"
                  disabled={loading}
                >
                  <CalendarIcon className="mr-2 h-4 w-4" />
                  {openedAsOf ? (
                    (formatUTCDateOnly(openedAsOf.toISOString()) ?? "")
                  ) : (
                    <span>{t("fields.selectDate")}</span>
                  )}
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-auto p-0">
                <Calendar mode="single" selected={openedAsOf} onSelect={setOpenedAsOf} />
              </PopoverContent>
            </Popover>
          </div>
        </div>

        {error && <p className="text-destructive text-sm">{error}</p>}

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("cancel")}
          </Button>
          <Button onClick={handleInit} loading={loading} disabled={!openedAsOf}>
            {t("confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
