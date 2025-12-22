"use client"

import DataTable, { Column } from "@lana/web/components/data-table"

import { Badge, BadgeProps } from "@lana/web/ui/badge"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { MeQuery, CreditFacilityStatus } from "@/lib/graphql/generated"

import { cn } from "@/lib/utils"
import Balance from "@/components/balance"

type CreditFacility = NonNullable<MeQuery["me"]["customer"]>["creditFacilities"][number]

type CustomerCreditFacilitiesTableProps = {
  creditFacilities: NonNullable<MeQuery["me"]["customer"]>["creditFacilities"]
}

export const CustomerCreditFacilitiesTable: React.FC<
  CustomerCreditFacilitiesTableProps
> = ({ creditFacilities }) => {
  const columns: Column<CreditFacility>[] = [
    {
      key: "activatedAt",
      header: "Activated At",
      render: (date) => <DateWithTooltip value={date} />,
    },
    {
      key: "balance",
      header: "Outstanding Balance",
      render: (_, facility) => (
        <Balance amount={facility.balance.outstanding.usdBalance} currency="usd" />
      ),
    },
    {
      key: "balance",
      header: "Collateral (BTC)",
      render: (_, facility) => (
        <Balance amount={facility.balance.collateral.btcBalance} currency="btc" />
      ),
    },
    {
      key: "collateralizationState",
      header: "Collateralization State",
      render: (state) =>
        state
          .toLowerCase()
          .split("_")
          .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
          .join(" "),
    },
    {
      key: "status",
      header: "Status",
      render: (status) => <LoanAndCreditFacilityStatusBadge status={status} />,
    },
  ]

  return (
    <DataTable
      data={creditFacilities}
      emptyMessage={
        <div className="min-h-40 w-full border rounded-md flex items-center justify-center">
          No Facility Found
        </div>
      }
      columns={columns}
      navigateTo={(facility) => `/credit-facilities/${facility.creditFacilityId}`}
    />
  )
}

interface LoanAndCreditFacilityStatusBadgeProps extends BadgeProps {
  status: CreditFacilityStatus
}

const getVariant = (status: CreditFacilityStatus) => {
  switch (status) {
    case CreditFacilityStatus.Active:
      return "success"
    case CreditFacilityStatus.Matured:
      return "default"
    case CreditFacilityStatus.Closed:
      return "secondary"
    default:
      return "secondary"
  }
}

export const LoanAndCreditFacilityStatusBadge = ({
  status,
  className,
  ...otherProps
}: LoanAndCreditFacilityStatusBadgeProps) => {
  const variant = getVariant(status)

  return (
    <Badge variant={variant} className={cn(className)} {...otherProps}>
      {status.split("_").join(" ")}
    </Badge>
  )
}
