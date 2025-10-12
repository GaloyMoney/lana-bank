"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { PendingCreditFacilityStatusBadge } from "./status-badge"
import { PendingCreditFacilityCollateralizationStateLabel } from "./label"

import {
  PendingCreditFacility,
  usePendingCreditFacilitiesQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"

gql`
  query PendingCreditFacilities($first: Int!, $after: String) {
    pendingCreditFacilities(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          pendingCreditFacilityId
          createdAt
          collateralizationState
          facilityAmount
          status
          collateral {
            btcBalance
          }
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

const PendingCreditFacilities = () => {
  const t = useTranslations("PendingCreditFacilities")

  const { data, loading, error, fetchMore } = usePendingCreditFacilitiesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error.message}</p>}
      <PaginatedTable<PendingCreditFacility>
        columns={columns(t)}
        data={data?.pendingCreditFacilities as PaginatedData<PendingCreditFacility>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(pending) =>
          `/pending-credit-facilities/${pending.pendingCreditFacilityId}`
        }
      />
    </div>
  )
}

export default PendingCreditFacilities

const columns = (t: (key: string) => string): Column<PendingCreditFacility>[] => [
  {
    key: "status",
    label: t("table.headers.status"),
    render: (status) => <PendingCreditFacilityStatusBadge status={status} />,
  },
  {
    key: "customer",
    label: t("table.headers.customer"),
    render: (customer) => customer.email,
  },
  {
    key: "facilityAmount",
    label: t("table.headers.facilityAmount"),
    render: (amount) => <Balance amount={amount} currency="usd" />,
  },
  {
    key: "collateral",
    label: t("table.headers.collateral"),
    render: (collateral) => <Balance amount={collateral.btcBalance} currency="btc" />,
  },
  {
    key: "collateralizationState",
    label: t("table.headers.collateralizationState"),
    render: (state) => <PendingCreditFacilityCollateralizationStateLabel state={state} />,
  },
  {
    key: "createdAt",
    label: t("table.headers.createdAt"),
    render: (date) => <DateWithTooltip value={date} />,
  },
]
