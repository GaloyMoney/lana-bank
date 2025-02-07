"use client"

import React from "react"

import DataTable, { Column } from "@lana/web/components/data-table"

import { Badge, BadgeProps } from "@lana/web/ui/badge"

import { GetCreditFacilityQuery, DisbursalStatus } from "@/lib/graphql/generated"
import { formatDate } from "@/lib/utils"

import Balance from "@/components/balance"

type Disbursal = NonNullable<
  NonNullable<GetCreditFacilityQuery["creditFacility"]>["disbursals"][number]
>

type CreditFacilityDisbursalsProps = {
  creditFacility: NonNullable<GetCreditFacilityQuery["creditFacility"]>
}

export const CreditFacilityDisbursals: React.FC<CreditFacilityDisbursalsProps> = ({
  creditFacility,
}) => {
  const columns: Column<Disbursal>[] = [
    {
      key: "amount",
      header: "Amount",
      render: (amount) => <Balance amount={amount} currency="usd" />,
    },
    {
      key: "createdAt",
      header: "Created At",
      render: (date) => formatDate(date),
    },
    {
      key: "status",
      header: "Status",
      align: "right",
      render: (_, disbursal) => {
        return <DisbursalStatusBadge status={disbursal.status} />
      },
    },
  ]

  return (
    <>
      <DataTable
        data={creditFacility.disbursals}
        columns={columns}
        emptyMessage={
          <div className="min-h-[10rem] w-full border rounded-md flex items-center justify-center">
            No Disbursals found
          </div>
        }
      />
    </>
  )
}

interface StatusBadgeProps extends BadgeProps {
  status: DisbursalStatus
}

const getVariant = (status: DisbursalStatus): BadgeProps["variant"] => {
  switch (status) {
    case DisbursalStatus.New:
      return "default"
    case DisbursalStatus.Approved:
      return "default"
    case DisbursalStatus.Confirmed:
      return "success"
    case DisbursalStatus.Denied:
      return "destructive"
    default:
      return "default"
  }
}

const DisbursalStatusBadge: React.FC<StatusBadgeProps> = ({ status, ...props }) => {
  const variant = getVariant(status)
  return (
    <Badge variant={variant} {...props}>
      {status}
    </Badge>
  )
}
