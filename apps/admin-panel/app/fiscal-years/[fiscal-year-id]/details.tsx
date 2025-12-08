"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"
import { CalendarCheck, CalendarPlus, CalendarX2 } from "lucide-react"

import { Button } from "@lana/web/ui/button"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@lana/web/ui/tooltip"
import { formatDate } from "@lana/web/utils"

import { FiscalYearStatusBadge } from "../status-badge"
import { CloseMonthDialog } from "../close-month"
import { CloseYearDialog } from "../close-year"
import { OpenNextYearDialog } from "../open-next-year"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetFiscalYearDetailsQuery, useFiscalYearsQuery } from "@/lib/graphql/generated"

type FiscalYearDetailsProps = {
  fiscalYear: NonNullable<GetFiscalYearDetailsQuery["fiscalYear"]>
}
const totalMonthsRequiredToCloseYear = 12

const FiscalYearDetailsCard: React.FC<FiscalYearDetailsProps> = ({ fiscalYear }) => {
  const t = useTranslations("FiscalYears.details")
  const [openCloseMonthDialog, setOpenCloseMonthDialog] = useState(false)
  const [openCloseYearDialog, setOpenCloseYearDialog] = useState(false)
  const [openOpenNextYearDialog, setOpenOpenNextYearDialog] = useState(false)
  const { data: latestFiscalYearData } = useFiscalYearsQuery({
    variables: { first: 1 },
    skip: fiscalYear.isOpen,
  })

  const latestFiscalYearId =
    latestFiscalYearData?.fiscalYears?.edges?.[0]?.node?.fiscalYearId

  const monthsClosed = fiscalYear.monthClosures.length
  const isCloseYearDisabled = monthsClosed < totalMonthsRequiredToCloseYear
  const lastClosure = monthsClosed > 0 ? fiscalYear.monthClosures[monthsClosed - 1] : null

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
      value: monthsClosed.toString(),
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
    <TooltipProvider>
      <div className="flex flex-wrap gap-2">
        {fiscalYear.isOpen && (
          <>
            {monthsClosed < totalMonthsRequiredToCloseYear && (
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
                  <p>
                    {t("buttons.closeYearDisabledTooltip", {
                      count: totalMonthsRequiredToCloseYear,
                    })}
                  </p>
                </TooltipContent>
              )}
            </Tooltip>
          </>
        )}
        {!fiscalYear.isOpen && fiscalYear.fiscalYearId === latestFiscalYearId && (
          <Button onClick={() => setOpenOpenNextYearDialog(true)}>
            <CalendarPlus className="h-4 w-4" />
            {t("buttons.openNextYear")}
          </Button>
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
      <OpenNextYearDialog
        fiscalYear={fiscalYear}
        open={openOpenNextYearDialog}
        onOpenChange={setOpenOpenNextYearDialog}
      />
    </>
  )
}

export default FiscalYearDetailsCard
