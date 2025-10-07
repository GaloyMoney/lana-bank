"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { Badge, BadgeProps } from "@lana/web/ui/badge"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import DataTable, { Column } from "@/components/data-table"

import {
  GetCreditFacilityRepaymentPlanQuery,
  GetCreditFacilityProposalRepaymentPlanQuery,
  GetPendingCreditFacilityRepaymentPlanQuery,
  CreditFacilityRepaymentStatus,
  CreditFacilityRepaymentType,
} from "@/lib/graphql/generated"

import Balance from "@/components/balance/balance"
import CardWrapper from "@/components/card-wrapper"

type RepaymentPlan = NonNullable<
  NonNullable<
    GetCreditFacilityRepaymentPlanQuery["creditFacilityByPublicId"]
  >["repaymentPlan"][number]
>

type CreditFacilityRepaymentPlanProps = {
  creditFacility: NonNullable<
    | GetCreditFacilityRepaymentPlanQuery["creditFacilityByPublicId"]
    | GetCreditFacilityProposalRepaymentPlanQuery["creditFacilityProposal"]
    | GetPendingCreditFacilityRepaymentPlanQuery["pendingCreditFacility"]
  >
}

export const CreditFacilityRepaymentPlan: React.FC<CreditFacilityRepaymentPlanProps> = ({
  creditFacility,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.RepaymentPlan")

  const getRepaymentTypeDisplay = (type: RepaymentPlan["repaymentType"]) => {
    switch (type) {
      case CreditFacilityRepaymentType.Disbursal:
        return t("repaymentTypes.principal")
      case CreditFacilityRepaymentType.Interest:
        return t("repaymentTypes.interest")
      default: {
        const exhaustiveCheck: never = type
        return exhaustiveCheck
      }
    }
  }

  const columns: Column<RepaymentPlan>[] = [
    {
      key: "status",
      header: t("columns.status"),
      render: (_, repayment) => {
        return <RepaymentStatusBadge status={repayment.status} t={t} />
      },
    },
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
    case CreditFacilityRepaymentStatus.Upcoming:
      return "default"
    case CreditFacilityRepaymentStatus.NotYetDue:
      return "outline"
    case CreditFacilityRepaymentStatus.Due:
      return "warning"
    case CreditFacilityRepaymentStatus.Overdue:
      return "destructive"
    case CreditFacilityRepaymentStatus.Defaulted:
      return "destructive"
    case CreditFacilityRepaymentStatus.Paid:
      return "success"
    default: {
      const exhaustiveCheck: never = status
      return exhaustiveCheck
    }
  }
}

const RepaymentStatusBadge: React.FC<StatusBadgeProps> = ({ status, t, ...props }) => {
  const variant = getStatusVariant(status)

  const getStatusKey = (status: RepaymentPlan["status"]): string => {
    switch (status) {
      case CreditFacilityRepaymentStatus.Upcoming:
        return "upcoming"
      case CreditFacilityRepaymentStatus.NotYetDue:
        return "notyetdue"
      case CreditFacilityRepaymentStatus.Due:
        return "due"
      case CreditFacilityRepaymentStatus.Overdue:
        return "overdue"
      case CreditFacilityRepaymentStatus.Defaulted:
        return "defaulted"
      case CreditFacilityRepaymentStatus.Paid:
        return "paid"
      default: {
        const exhaustiveCheck: never = status
        return exhaustiveCheck
      }
    }
  }

  return (
    <Badge variant={variant} {...props}>
      {t(`status.${getStatusKey(status)}`)}
    </Badge>
  )
}
