"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { CreditFacilityProposalStatusBadge } from "./status-badge"

import {
  CreditFacilityProposal,
  useCreditFacilityProposalsQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"

gql`
  query CreditFacilityProposals($first: Int!, $after: String) {
    creditFacilityProposals(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          creditFacilityProposalId
          createdAt
          facilityAmount
          status
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }
`

const CreditFacilityProposals = () => {
  const t = useTranslations("CreditFacilityProposals")
  const commonT = useTranslations("Common")

  const { data, loading, error, fetchMore } = useCreditFacilityProposalsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{commonT("error")}</p>}
      <PaginatedTable<CreditFacilityProposal>
        columns={columns(t)}
        data={data?.creditFacilityProposals as PaginatedData<CreditFacilityProposal>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(proposal) =>
          `/credit-facility-proposals/${proposal.creditFacilityProposalId}`
        }
      />
    </div>
  )
}

export default CreditFacilityProposals

const columns = (t: (key: string) => string): Column<CreditFacilityProposal>[] => [
  {
    key: "status",
    label: t("table.headers.status"),
    labelClassName: "w-[20%]",
    render: (status) => <CreditFacilityProposalStatusBadge status={status} />,
  },
  {
    key: "facilityAmount",
    label: t("table.headers.facilityAmount"),
    labelClassName: "w-[40%]",
    render: (amount) => <Balance amount={amount} currency="usd" />,
  },
  {
    key: "createdAt",
    label: t("table.headers.createdAt"),
    labelClassName: "w-[25%]",
    render: (date) => <DateWithTooltip value={date} />,
  },
]
