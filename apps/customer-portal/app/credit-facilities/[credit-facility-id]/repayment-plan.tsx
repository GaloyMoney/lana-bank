"use client"

import React from "react"

import DataTable, { Column } from "@lana/web/components/data-table"

import { Badge, BadgeProps } from "@lana/web/ui/badge"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { GetCreditFacilityQuery } from "@/lib/graphql/generated"

import Balance from "@/components/balance"

type RepaymentPlan = NonNullable<
  NonNullable<GetCreditFacilityQuery["creditFacility"]>["repaymentPlan"][number]
>

type CreditFacilityRepaymentPlanProps = {
  creditFacility: NonNullable<GetCreditFacilityQuery["creditFacility"]>
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
    {
      key: "status",
      header: "Status",
      align: "right",
      render: (_, repayment) => {
        return <RepaymentStatusBadge status={repayment.status} />
      },
    },
  ]

  const repaymentPlanData = creditFacility?.repaymentPlan ?? []

  return (
    <DataTable
      data={repaymentPlanData}
      columns={columns}
      emptyMessage={
        <div className="min-h-[10rem] w-full border rounded-md flex items-center justify-center">
          No Plan found
        </div>
      }
    />
  )
}

interface StatusBadgeProps extends BadgeProps {
  status: RepaymentPlan["status"]
}

const getStatusVariant = (status: RepaymentPlan["status"]): BadgeProps["variant"] => {
  switch (status) {
    case "UPCOMING":
      return "default"
    case "NOT_YET_DUE":
      return "outline"
    case "DUE":
      return "warning"
    case "OVERDUE":
      return "destructive"
    case "PAID":
      return "success"
    default:
      return "default"
  }
}

const RepaymentStatusBadge: React.FC<StatusBadgeProps> = ({ status, ...props }) => {
  const variant = getStatusVariant(status)
  return (
    <Badge variant={variant} {...props}>
      {status}
    </Badge>
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
