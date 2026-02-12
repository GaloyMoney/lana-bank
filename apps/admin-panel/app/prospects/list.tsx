"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { KycStatusBadge } from "./kyc-status-badge"

import {
  KycStatus,
  Prospect,
  useProspectsQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

gql`
  query Prospects($first: Int!, $after: String) {
    prospects(first: $first, after: $after) {
      edges {
        node {
          id
          prospectId
          publicId
          kycStatus
          level
          email
          telegramHandle
          applicantId
          customerType
          createdAt
        }
        cursor
      }
      pageInfo {
        endCursor
        startCursor
        hasNextPage
        hasPreviousPage
      }
    }
  }
`

const ProspectsList = () => {
  const t = useTranslations("Prospects")

  const { data, loading, error, fetchMore } = useProspectsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  const columns: Column<Prospect>[] = [
    {
      key: "email",
      label: t("columns.email"),
      labelClassName: "w-[30%]",
    },
    {
      key: "telegramHandle",
      label: t("columns.telegramHandle"),
      labelClassName: "w-[30%]",
    },
    {
      key: "kycStatus",
      label: t("columns.kycStatus"),
      filterValues: Object.values(KycStatus),
      render: (status) => <KycStatusBadge status={status} />,
    },
    {
      key: "customerType",
      label: t("columns.customerType"),
    },
  ]

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Prospect>
        columns={columns}
        data={data?.prospects as PaginatedData<Prospect>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(prospect) => `/prospects/${prospect.publicId}`}
      />
    </div>
  )
}

export default ProspectsList
