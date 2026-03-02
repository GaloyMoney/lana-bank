"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { CreditFacilityProposalStatusBadge } from "./status-badge"

import {
  CreditFacilityProposal,
  CreditFacilityProposalStatus,
  CreditFacilityProposalsFilter,
  useCreditFacilityProposalsQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"

gql`
  query CreditFacilityProposals($first: Int!, $after: String, $filter: CreditFacilityProposalsFilter) {
    creditFacilityProposals(first: $first, after: $after, filter: $filter) {
      edges {
        cursor
        node {
          id
          creditFacilityProposalId
          createdAt
          facilityAmount
          status
          customer {
            customerId
            email
          }
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
  const [filter, setFilter] = useState<CreditFacilityProposalsFilter | null>(null)

  const { data, loading, error, fetchMore } = useCreditFacilityProposalsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      filter,
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
        onFilter={(filters) => {
          const f = filters as CreditFacilityProposalsFilter
          setFilter(Object.keys(f).length > 0 ? f : null)
        }}
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
    filterValues: Object.values(CreditFacilityProposalStatus),
  },
  {
    key: "customer",
    label: t("table.headers.customer"),
    labelClassName: "w-[40%]",
    render: (customer) => <div className="truncate">{customer.email}</div>,
  },
  {
    key: "facilityAmount",
    label: t("table.headers.facilityAmount"),
    labelClassName: "w-[25%]",
    render: (amount) => <Balance amount={amount} currency="usd" />,
  },
  {
    key: "createdAt",
    label: t("table.headers.createdAt"),
    labelClassName: "w-[15%]",
    render: (date) => <DateWithTooltip value={date} />,
  },
]
