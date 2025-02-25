"use client"

import { useTranslations } from "next-intl"

import CardWrapper from "@/components/card-wrapper"
import Balance from "@/components/balance/balance"
import { GetCustomerCreditFacilitiesQuery } from "@/lib/graphql/generated"
import { formatCollateralizationState, formatDate } from "@/lib/utils"
import { LoanAndCreditFacilityStatusBadge } from "@/app/loans/status-badge"
import DataTable, { Column } from "@/components/data-table"

type CreditFacility = NonNullable<
  GetCustomerCreditFacilitiesQuery["customer"]
>["creditFacilities"][number]

type CustomerCreditFacilitiesTableProps = {
  creditFacilities: NonNullable<
    GetCustomerCreditFacilitiesQuery["customer"]
  >["creditFacilities"]
}

export const CustomerCreditFacilitiesTable: React.FC<
  CustomerCreditFacilitiesTableProps
> = ({ creditFacilities }) => {
  const t = useTranslations("Customers.CustomerDetails.creditFacilities")

  const columns: Column<CreditFacility>[] = [
    {
      key: "status",
      header: t("table.headers.status"),
      render: (status) => <LoanAndCreditFacilityStatusBadge status={status} />,
    },
    {
      key: "balance",
      header: t("table.headers.outstandingBalance"),
      render: (_, facility) => (
        <Balance amount={facility.balance.outstanding.usdBalance} currency="usd" />
      ),
    },
    {
      key: "balance",
      header: t("table.headers.collateralBtc"),
      render: (_, facility) => (
        <Balance amount={facility.balance.collateral.btcBalance} currency="btc" />
      ),
    },
    {
      key: "collateralizationState",
      header: t("table.headers.collateralizationState"),
      render: (state) => formatCollateralizationState(state),
    },
    {
      key: "createdAt",
      header: t("table.headers.createdAt"),
      render: (date) => formatDate(date, { includeTime: false }),
    },
  ]

  return (
    <CardWrapper title={t("title")} description={t("description")}>
      <DataTable
        data={creditFacilities}
        columns={columns}
        navigateTo={(facility) => `/credit-facilities/${facility.creditFacilityId}`}
      />
    </CardWrapper>
  )
}
