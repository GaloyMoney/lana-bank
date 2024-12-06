"use client"

import React from "react"

import { GetCreditFacilityDetailsQuery } from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { formatDate, formatInterval, formatPeriod } from "@/lib/utils"
import { DetailsCard, DetailItemProps } from "@/components/details"

type CreditFacilityTermsProps = {
  creditFacility: NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>
}

const CreditFacilityTerms: React.FC<CreditFacilityTermsProps> = ({ creditFacility }) => {
  const details: DetailItemProps[] = [
    {
      label: "Duration",
      value: `${creditFacility.creditFacilityTerms.duration.units} ${formatPeriod(
        creditFacility.creditFacilityTerms.duration.period,
      )}`,
    },
    {
      label: "Interest (APR)",
      value: `${creditFacility.creditFacilityTerms.annualRate}%`,
    },
    {
      label: "Accrual Interval",
      value: formatInterval(creditFacility.creditFacilityTerms.accrualInterval),
    },
    {
      label: "Target/initial CVL %",
      value: `${creditFacility.creditFacilityTerms.initialCvl}%`,
    },
    {
      label: "Margin call CVL %",
      value: `${creditFacility.creditFacilityTerms.marginCallCvl}%`,
    },
    {
      label: "Liquidation CVL %",
      value: `${creditFacility.creditFacilityTerms.liquidationCvl}%`,
    },
    {
      label: "Date created",
      value: formatDate(creditFacility.createdAt),
    },
    {
      label: "Facility Amount",
      value: <Balance amount={creditFacility.facilityAmount} currency="usd" />,
    },
    {
      label: "Incurrence Interval",
      value: formatInterval(creditFacility.creditFacilityTerms.incurrenceInterval),
    },
    {
      label: "One Time Fee Rate",
      value: `${creditFacility.creditFacilityTerms.oneTimeFeeRate}%`,
    },
  ]

  return (
    <DetailsCard
      title="Credit Facility Terms"
      details={details}
      description="Terms Details for this credit facility"
    />
  )
}

export { CreditFacilityTerms }
