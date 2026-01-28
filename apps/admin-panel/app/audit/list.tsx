"use client"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import { AuditEntry, useAuditLogsQuery } from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

gql`
  query AuditLogs($first: Int!, $after: String) {
    audit(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          auditEntryId
          subject {
            ... on User {
              userId
              email
              role {
                roleId
                name
              }
            }
            ... on System {
              name
            }
          }
          object
          action
          authorized
          recordedAt
        }
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

const AuditLogsList = () => {
  const t = useTranslations("AuditLogs.table")

  const { data, loading, error, fetchMore } = useAuditLogsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
    fetchPolicy: "cache-and-network",
  })

  const columns: Column<AuditEntry>[] = [
    {
      key: "auditEntryId",
      label: t("headers.auditEntryId"),
      labelClassName: "w-[10%]",
    },
    {
      key: "subject",
      label: t("headers.subject"),
      labelClassName: "w-[20%]",
      render: (subject) => (
        <div>{subject.__typename === "User" ? subject.email : subject.__typename}</div>
      ),
    },
    {
      key: "object",
      label: t("headers.object"),
      labelClassName: "w-[35%]",
    },
    {
      key: "action",
      label: t("headers.action"),
      labelClassName: "w-[20%]",
    },
    {
      key: "authorized",
      label: t("headers.authorized"),
      labelClassName: "w-[10%]",
      render: (authorized) => (
        <span className={authorized ? "text-green-600" : "text-red-600 font-semibold"}>
          {authorized ? "Yes" : "No"}
        </span>
      ),
    },
    {
      key: "recordedAt",
      label: t("headers.recordedAt"),
      labelClassName: "w-[15%]",
      render: (date) => formatDate(date),
    },
  ]

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<AuditEntry>
        columns={columns}
        data={data?.audit as PaginatedData<AuditEntry>}
        loading={loading}
        pageSize={DEFAULT_PAGESIZE}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
      />
    </div>
  )
}

export default AuditLogsList
