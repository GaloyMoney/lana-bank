"use client"

import { useTranslations } from "next-intl"
import { gql } from "@apollo/client"

import { Policy, usePoliciesQuery } from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { formatProcessType } from "@/lib/utils"

gql`
  query Policies($first: Int!, $after: String) {
    policies(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          policyId
          approvalProcessType
          rules {
            ... on CommitteeThreshold {
              threshold
              committee {
                id
                committeeId
                createdAt
                name
              }
            }
            ... on SystemApproval {
              autoApprove
            }
          }
        }
      }
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
    }
  }
`

const PolicyList = () => {
  const t = useTranslations("Policies.table")

  const { data, loading, error, fetchMore } = usePoliciesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{t("errors.general")}</p>}
      <PaginatedTable
        columns={columns(t)}
        data={data?.policies as PaginatedData<Policy>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(policy) => `/policies/${policy.policyId}`}
      />
    </div>
  )
}

export default PolicyList

const columns = (t: ReturnType<typeof useTranslations>): Column<Policy>[] => [
  {
    key: "approvalProcessType",
    label: t("headers.approvalProcessType"),
    render: (type) => formatProcessType(type),
  },
  {
    key: "rules",
    label: t("headers.rules"),
    render: (rules) => {
      if (rules.__typename === "CommitteeThreshold") {
        return t("rules.committeeThreshold", { committeeName: rules.committee.name })
      }
      if (rules.__typename === "SystemApproval") {
        return <span className="text-textColor-secondary">{t("rules.autoApprove")}</span>
      }
      return ""
    },
  },
]
