"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { Badge, BadgeProps } from "@lana/web/ui/badge"

import DataTable, { Column } from "@/components/data-table"

import { GetCreditFacilityRepaymentPlanQuery } from "@/lib/graphql/generated"

import Balance from "@/components/balance/balance"
import CardWrapper from "@/components/card-wrapper"
import DateWithTooltip from "@lana/web/components/date-with-tooltip"

type RepaymentPlan = NonNullable<
  NonNullable<
    GetCreditFacilityRepaymentPlanQuery["creditFacility"]
  >["repaymentPlan"][number]
>

type CreditFacilityRepaymentPlanProps = {
  creditFacility: NonNullable<GetCreditFacilityRepaymentPlanQuery["creditFacility"]>
}

export const CreditFacilityRepaymentPlan: React.FC<CreditFacilityRepaymentPlanProps> = ({
  creditFacility,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.RepaymentPlan")

  const getRepaymentTypeDisplay = (type: RepaymentPlan["repaymentType"]) => {
    switch (type) {
      case "DISBURSAL":
        return t("repaymentTypes.principal")
      case "INTEREST":
        return t("repaymentTypes.interest")
      default:
        return type
    }
  }

  const columns: Column<RepaymentPlan>[] = [
    {
      key: "repaymentType",
      header: t("columns.type"),
      render: (type) => getRepaymentTypeDisplay(type),
    },
    {
      key: "initial",
      header: t("columns.initialAmount"),
      render: (amount) => <Balance amount={amount} currency="usd" />,
    },
    {
      key: "outstanding",
      header: t("columns.outstanding"),
      render: (amount) => <Balance amount={amount} currency="usd" />,
    },
    {
      key: "dueAt",
      header: t("columns.dueDate"),
      render: (date) => <DateWithTooltip value={date} />,
    },
    {
      key: "status",
      header: t("columns.status"),
      align: "right",
      render: (_, repayment) => {
        return <RepaymentStatusBadge status={repayment.status} t={t} />
      },
    },
  ]

  const repaymentPlanData = creditFacility?.repaymentPlan ?? []

  return (
    <CardWrapper title={t("title")} description={t("description")}>
      <DataTable
        data={repaymentPlanData}
        columns={columns}
        emptyMessage={t("messages.emptyTable")}
      />
    </CardWrapper>
  )
}

interface StatusBadgeProps extends BadgeProps {
  status: RepaymentPlan["status"]
  t: (key: string) => string
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

const RepaymentStatusBadge: React.FC<StatusBadgeProps> = ({ status, t, ...props }) => {
  const variant = getStatusVariant(status)
  const statusKey = status.toLowerCase()

  return (
    <Badge variant={variant} {...props}>
      {t(`status.${statusKey}`)}
    </Badge>
  )
}
