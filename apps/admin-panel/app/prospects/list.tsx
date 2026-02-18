"use client"

import { useState } from "react"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { ProspectStageBadge } from "./prospect-stage-badge"

import { Prospect, ProspectStage, useProspectsQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

gql`
  query Prospects($first: Int!, $after: String, $stage: ProspectStage) {
    prospects(first: $first, after: $after, stage: $stage) {
      edges {
        node {
          id
          prospectId
          publicId
          stage
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
  const [stageFilter, setStageFilter] = useState<ProspectStage | undefined>(undefined)

  const { data, loading, error, fetchMore } = useProspectsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      stage: stageFilter,
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
      key: "stage",
      label: t("columns.stage"),
      filterValues: [
        ProspectStage.New,
        ProspectStage.KycStarted,
        ProspectStage.KycPending,
        ProspectStage.KycDeclined,
        ProspectStage.Converted,
        ProspectStage.Closed,
      ],
      render: (stage) => <ProspectStageBadge stage={stage} />,
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
        onFilter={(filters) => {
          setStageFilter(filters.stage as ProspectStage)
        }}
      />
    </div>
  )
}

export default ProspectsList
