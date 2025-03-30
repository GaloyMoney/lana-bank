import { DetailItemProps, DetailsCard } from "@lana/web/components/details"

import React from "react"

import { CreditFacility } from "@/lib/graphql/generated"
import { formatPeriod, removeUnderscore } from "@/lib/kratos/utils"

function TermsCard({ data }: { data: NonNullable<CreditFacility> }) {
  const terms: DetailItemProps[] = [
    {
      label: "Duration",
      value: `${data.creditFacilityTerms.duration.units} ${formatPeriod(
        data.creditFacilityTerms.duration.period,
      )}`,
    },
    {
      label: "Interest (APR)",
      value: `${data.creditFacilityTerms.annualRate}%`,
    },
    {
      label: "Accrual Cycle Interval",
      value: removeUnderscore(data.creditFacilityTerms.accrualCycleInterval),
    },
    {
      label: "Target/initial CVL %",
      value: `${data.creditFacilityTerms.initialCvl}%`,
    },
    {
      label: "Margin call CVL %",
      value: `${data.creditFacilityTerms.marginCallCvl}%`,
    },
    {
      label: "Liquidation CVL %",
      value: `${data.creditFacilityTerms.liquidationCvl}%`,
    },
    {
      label: "Accrual Interval",
      value: removeUnderscore(data.creditFacilityTerms.accrualInterval),
    },
    {
      label: "Structuring Fee Rate",
      value: `${data.creditFacilityTerms.oneTimeFeeRate}%`,
    },
  ]
  return <DetailsCard className="w-full" title="Terms" details={terms} />
}

export default TermsCard
