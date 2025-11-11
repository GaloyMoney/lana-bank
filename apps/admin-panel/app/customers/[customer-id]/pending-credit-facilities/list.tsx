"use client"

import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import CardWrapper from "@/components/card-wrapper"
import Balance from "@/components/balance/balance"
import { GetCustomerPendingCreditFacilitiesQuery } from "@/lib/graphql/generated"

import { PendingCreditFacilityStatusBadge } from "@/app/pending-credit-facilities/status-badge"
import { PendingFacilityCollateralizationStateLabel } from "@/app/pending-credit-facilities/label"
import DataTable, { Column } from "@/components/data-table"

type PendingCreditFacility = NonNullable<
  GetCustomerPendingCreditFacilitiesQuery["customerByPublicId"]
>["pendingCreditFacilities"][number]

type CustomerPendingCreditFacilitiesTableProps = {
  pendingCreditFacilities: NonNullable<
    GetCustomerPendingCreditFacilitiesQuery["customerByPublicId"]
  >["pendingCreditFacilities"]
}

export const CustomerPendingCreditFacilitiesTable: React.FC<
  CustomerPendingCreditFacilitiesTableProps
> = ({ pendingCreditFacilities }) => {
  const t = useTranslations("Customers.CustomerDetails.pendingCreditFacilities")

  const columns: Column<PendingCreditFacility>[] = [
    {
      key: "status",
      header: t("table.headers.status"),
      render: (status) => <PendingCreditFacilityStatusBadge status={status} />,
    },
    {
      key: "facilityAmount",
      header: t("table.headers.facilityAmount"),
      render: (amount) => <Balance amount={amount} currency="usd" />,
    },
    {
      key: "collateral",
      header: t("table.headers.collateral"),
      render: (_, pending) => (
        <Balance amount={pending.collateral.btcBalance} currency="btc" />
      ),
    },
    {
      key: "collateralizationState",
      header: t("table.headers.collateralizationState"),
      render: (state) => <PendingFacilityCollateralizationStateLabel state={state} />,
    },
    {
      key: "createdAt",
      header: t("table.headers.createdAt"),
      render: (date) => <DateWithTooltip value={date} />,
    },
  ]

  return (
    <CardWrapper title={t("title")} description={t("description")}>
      <DataTable
        data={pendingCreditFacilities}
        columns={columns}
        navigateTo={(pending) =>
          `/pending-credit-facilities/${pending.pendingCreditFacilityId}`
        }
      />
    </CardWrapper>
  )
}
