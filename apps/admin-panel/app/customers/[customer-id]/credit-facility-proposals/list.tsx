"use client"

import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import CardWrapper from "@/components/card-wrapper"
import Balance from "@/components/balance/balance"
import { GetCustomerCreditFacilityProposalsQuery } from "@/lib/graphql/generated"

import { CreditFacilityProposalStatusBadge } from "@/app/credit-facility-proposals/status-badge"
import DataTable, { Column } from "@/components/data-table"

type CreditFacilityProposal = NonNullable<
  GetCustomerCreditFacilityProposalsQuery["customerByPublicId"]
>["creditFacilityProposals"][number]

type CustomerCreditFacilityProposalsTableProps = {
  creditFacilityProposals: NonNullable<
    GetCustomerCreditFacilityProposalsQuery["customerByPublicId"]
  >["creditFacilityProposals"]
}

export const CustomerCreditFacilityProposalsTable: React.FC<
  CustomerCreditFacilityProposalsTableProps
> = ({ creditFacilityProposals }) => {
  const t = useTranslations("Customers.CustomerDetails.creditFacilityProposals")

  const columns: Column<CreditFacilityProposal>[] = [
    {
      key: "status",
      header: t("table.headers.status"),
      render: (status) => <CreditFacilityProposalStatusBadge status={status} />,
    },
    {
      key: "facilityAmount",
      header: t("table.headers.facilityAmount"),
      render: (amount) => <Balance amount={amount} currency="usd" />,
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
        data={creditFacilityProposals}
        columns={columns}
        navigateTo={(proposal) =>
          `/credit-facility-proposals/${proposal.creditFacilityProposalId}`
        }
      />
    </CardWrapper>
  )
}
