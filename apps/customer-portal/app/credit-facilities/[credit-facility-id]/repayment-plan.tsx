"use client"

import React from "react"

import DataTable, { Column } from "@lana/web/components/data-table"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { GetCreditFacilityQuery } from "@/lib/graphql/generated"

import Balance from "@/components/balance"

type CreditFacilityData = NonNullable<GetCreditFacilityQuery["creditFacility"]>

type RepaymentPlan = NonNullable<CreditFacilityData["repaymentPlan"][number]>

type CreditFacilityRepaymentPlanProps = {
  creditFacility: CreditFacilityData
}

export const CreditFacilityRepaymentPlan: React.FC<CreditFacilityRepaymentPlanProps> = ({
  creditFacility,
}) => {
  const columns: Column<RepaymentPlan>[] = [
    {
      key: "repaymentType",
      header: "Type",
      render: (type) => getRepaymentTypeDisplay(type),
    },
    {
      key: "initial",
      header: "Initial Amount",
      render: (amount) => <Balance amount={amount} currency="usd" />,
    },
    {
      key: "outstanding",
      header: "Outstanding",
      render: (amount) => <Balance amount={amount} currency="usd" />,
    },
    {
      key: "dueAt",
      header: "Due Date",
      render: (date) => <DateWithTooltip value={date} />,
    },
  ]

  const repaymentPlanData = creditFacility?.repaymentPlan ?? []

  return (
    <DataTable
      data={repaymentPlanData}
      columns={columns}
      emptyMessage={
        <div className="min-h-40 w-full border rounded-md flex items-center justify-center">
          No Plan found
        </div>
      }
    />
  )
}

const getRepaymentTypeDisplay = (type: RepaymentPlan["repaymentType"]) => {
  switch (type) {
    case "DISBURSAL":
      return "Principal"
    case "INTEREST":
      return "Interest"
    default:
      return type
  }
}
