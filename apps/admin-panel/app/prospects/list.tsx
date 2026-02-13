"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { KycStatusBadge } from "./kyc-status-badge"
import { ProspectStatusBadge } from "./prospect-status-badge"

import {
  KycStatus,
  Prospect,
  ProspectStatus,
  useProspectsQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

gql`
  query Prospects($first: Int!, $after: String, $status: ProspectStatus) {
    prospects(first: $first, after: $after, status: $status) {
      edges {
        node {
          id
          prospectId
          publicId
          status
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
      status: ProspectStatus.Open,
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
      key: "status",
      label: t("columns.status"),
      filterValues: Object.values(ProspectStatus),
      render: (status) => <ProspectStatusBadge status={status} />,
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
