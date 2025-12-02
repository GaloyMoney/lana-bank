"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"
import { CalendarCheck } from "lucide-react"

import { Button } from "@lana/web/ui/button"
import { formatDate } from "@lana/web/utils"

import { FiscalYearStatusBadge } from "../status-badge"
import { CloseMonthDialog } from "../close-month"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetFiscalYearDetailsQuery } from "@/lib/graphql/generated"

type FiscalYearDetailsProps = {
  fiscalYear: NonNullable<GetFiscalYearDetailsQuery["fiscalYear"]>
}

const FiscalYearDetailsCard: React.FC<FiscalYearDetailsProps> = ({ fiscalYear }) => {
  const t = useTranslations("FiscalYears.details")
  const [openCloseMonthDialog, setOpenCloseMonthDialog] = useState(false)

  const lastClosure =
    fiscalYear.monthClosures.length > 0
      ? fiscalYear.monthClosures[fiscalYear.monthClosures.length - 1]
      : null

  const details: DetailItemProps[] = [
    {
      label: t("fields.openedAsOf"),
      value: formatDate(fiscalYear.openedAsOf),
    },
    {
      label: t("fields.status"),
      value: <FiscalYearStatusBadge isOpen={fiscalYear.isOpen} />,
    },
    {
      label: t("fields.monthsClosed"),
      value: fiscalYear.monthClosures.length.toString(),
    },
    {
      label: t("fields.lastClosedMonth"),
      value: lastClosure ? (
        <div className="flex flex-col gap-0.5">
          <span>{formatDate(lastClosure.closedAsOf)}</span>
          <span className="text-xs text-muted-foreground">
            {t("fields.processedAt", { date: formatDate(lastClosure.closedAt) })}
          </span>
        </div>
      ) : (
        t("fields.noMonthsClosed")
      ),
    },
  ]

  const footerContent = (
    <Button variant="outline" onClick={() => setOpenCloseMonthDialog(true)}>
      <CalendarCheck className="h-4 w-4" />
      {t("buttons.closeMonth")}
    </Button>
  )

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        footerContent={footerContent}
        className="max-w-7xl m-auto"
      />
      <CloseMonthDialog
        fiscalYear={fiscalYear}
        open={openCloseMonthDialog}
        onOpenChange={setOpenCloseMonthDialog}
      />
    </>
  )
}

export default FiscalYearDetailsCard
