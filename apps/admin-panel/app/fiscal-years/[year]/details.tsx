"use client"

import React, { useState } from "react"
import { useLocale, useTranslations } from "next-intl"
import { CalendarCheck, CalendarX2 } from "lucide-react"

import { Button } from "@lana/web/ui/button"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@lana/web/ui/tooltip"
import { formatDate, formatUTCDateOnly, formatUTCMonthYear } from "@lana/web/utils"

import { FiscalYearStatusBadge } from "../status-badge"
import { CloseMonthDialog } from "../close-month"
import { CloseYearDialog } from "../close-year"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetFiscalYearDetailsQuery } from "@/lib/graphql/generated"

type FiscalYearDetailsProps = {
  fiscalYear: NonNullable<GetFiscalYearDetailsQuery["fiscalYearByYear"]>
}
const FiscalYearDetailsCard: React.FC<FiscalYearDetailsProps> = ({ fiscalYear }) => {
  const t = useTranslations("FiscalYears.details")
  const locale = useLocale()
  const [openCloseMonthDialog, setOpenCloseMonthDialog] = useState(false)
  const [openCloseYearDialog, setOpenCloseYearDialog] = useState(false)

  const monthsClosed = fiscalYear.monthClosures.length
  const isCloseYearDisabled = !fiscalYear.isLastMonthOfYearClosed
  const lastClosure = monthsClosed > 0 ? fiscalYear.monthClosures[monthsClosed - 1] : null

  const nextMonthToCloseDisplay = formatUTCMonthYear(
    fiscalYear.nextMonthToClose,
    locale,
  )

  const details: DetailItemProps[] = [
    {
      label: t("fields.openedAsOf"),
      value: formatUTCDateOnly(fiscalYear.openedAsOf) ?? "-",
    },
    {
      label: t("fields.status"),
      value: <FiscalYearStatusBadge isOpen={fiscalYear.isOpen} />,
    },
    {
      label: t("fields.monthsClosed"),
      value: monthsClosed.toString(),
    },
    {
      label: t("fields.lastClosedMonth"),
      value: lastClosure ? (
        <div className="flex flex-col gap-0.5">
          <span>{formatUTCDateOnly(lastClosure.closedAsOf) ?? "-"}</span>
          <span className="text-xs text-muted-foreground">
            {t("fields.processedAt", { date: formatDate(lastClosure.closedAt) })}
          </span>
        </div>
      ) : (
        t("fields.noMonthsClosed")
      ),
    },
    {
      label: t("fields.nextMonthToClose"),
      value: nextMonthToCloseDisplay ?? t("fields.allMonthsClosed"),
    },
  ]

  const footerContent = (
    <TooltipProvider>
      <div className="flex flex-wrap gap-2">
        {fiscalYear.isOpen && (
          <>
            {!fiscalYear.isLastMonthOfYearClosed && (
              <Button variant="outline" onClick={() => setOpenCloseMonthDialog(true)}>
                <CalendarCheck className="h-4 w-4" />
                {t("buttons.closeMonth")}
              </Button>
            )}
            <Tooltip>
              <TooltipTrigger asChild>
                <div>
                  <Button
                    variant="outline"
                    onClick={() => setOpenCloseYearDialog(true)}
                    disabled={isCloseYearDisabled}
                  >
                    <CalendarX2 className="h-4 w-4" />
                    {t("buttons.closeYear")}
                  </Button>
                </div>
              </TooltipTrigger>
              {isCloseYearDisabled && (
                <TooltipContent>
                  <p>{t("buttons.closeYearDisabledTooltip")}</p>
                </TooltipContent>
              )}
            </Tooltip>
          </>
        )}
      </div>
    </TooltipProvider>
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
      <CloseYearDialog
        fiscalYear={fiscalYear}
        open={openCloseYearDialog}
        onOpenChange={setOpenCloseYearDialog}
      />
    </>
  )
}

export default FiscalYearDetailsCard
