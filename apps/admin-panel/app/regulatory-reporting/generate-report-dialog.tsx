"use client"

import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Label } from "@lana/web/ui/label"

import { AsOfDateSelector } from "@/components/as-of-date-selector"

type GenerateReportDialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  reportName: string | null
  asOfDate: string
  onAsOfDateChange: (date: string) => void
  onGenerate: () => void
  generating: boolean
}

const GenerateReportDialog: React.FC<GenerateReportDialogProps> = ({
  open,
  onOpenChange,
  reportName,
  asOfDate,
  onAsOfDateChange,
  onGenerate,
  generating,
}) => {
  const t = useTranslations("Reports")

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[480px]">
        <DialogHeader>
          <DialogTitle>{t("ReportGeneration.selectAsOfDate")}</DialogTitle>
          <DialogDescription>
            {reportName
              ? t("ReportGeneration.selectAsOfDateDescription", {
                  report: reportName,
                })
              : null}
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-3">
          <div className="flex flex-col gap-2">
            <Label>{t("ReportGeneration.asOfDate")}</Label>
            <AsOfDateSelector asOf={asOfDate} onDateChange={onAsOfDateChange} />
          </div>
        </div>
        <DialogFooter>
          <Button
            type="button"
            disabled={!open || generating}
            onClick={onGenerate}
          >
            {t("ReportGeneration.generate")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export { GenerateReportDialog }
