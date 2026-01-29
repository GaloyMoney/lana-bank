"use client"
import { useCallback, useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { useRouter } from "next/navigation"

import { formatDate } from "@lana/web/utils"

import {
  AuditEntry,
  useAuditLogsQuery,
  useAuditSubjectsQuery,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

gql`
  query AuditLogs($first: Int!, $after: String, $subject: String, $authorized: Boolean, $object: String, $action: String) {
    audit(first: $first, after: $after, subject: $subject, authorized: $authorized, object: $object, action: $action) {
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

gql`
  query AuditSubjects {
    auditSubjects
  }
`

interface AuditLogsListProps {
  page?: number
}

const AuditLogsList = ({ page = 1 }: AuditLogsListProps) => {
  const t = useTranslations("AuditLogs.table")
  const router = useRouter()

  const [subjectFilter, setSubjectFilter] = useState<string | undefined>(undefined)
  const [authorizedFilter, setAuthorizedFilter] = useState<boolean | undefined>(
    undefined,
  )
  const [objectFilter, setObjectFilter] = useState<string | undefined>(undefined)
  const [actionFilter, setActionFilter] = useState<string | undefined>(undefined)

  const { data, loading, error, fetchMore } = useAuditLogsQuery({
    variables: {
      first: DEFAULT_PAGESIZE * page,
      subject: subjectFilter ?? null,
      authorized: authorizedFilter ?? null,
      object: objectFilter ?? null,
      action: actionFilter ?? null,
    },
    fetchPolicy: "cache-and-network",
  })

  const { data: subjectsData } = useAuditSubjectsQuery({
    fetchPolicy: "cache-and-network",
  })

  const handlePageChange = useCallback(
    (newPage: number) => {
      if (newPage === 1) {
        router.push("/audit")
      } else {
        router.push(`/audit/${newPage}`)
      }
    },
    [router],
  )

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
      render: (subject) => {
        if (subject.__typename === "User") {
          return <div>user: {subject.email}</div>
        }
        if (subject.__typename === "System") {
          return <div>system</div>
        }
        return <div>{subject.__typename}</div>
      },
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
          {authorized ? t("headers.authorizedYes") : t("headers.authorizedNo")}
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
      <div className="flex gap-2 mb-4">
        <select
          className="border rounded px-2 py-1 text-sm"
          value={subjectFilter ?? ""}
          onChange={(e) => setSubjectFilter(e.target.value || undefined)}
        >
          <option value="">{t("filters.allSubjects")}</option>
          {subjectsData?.auditSubjects.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </select>
        <select
          className="border rounded px-2 py-1 text-sm"
          value={authorizedFilter === undefined ? "" : String(authorizedFilter)}
          onChange={(e) => {
            const val = e.target.value
            setAuthorizedFilter(val === "" ? undefined : val === "true")
          }}
        >
          <option value="">{t("filters.allAuthorized")}</option>
          <option value="true">{t("filters.authorizedOnly")}</option>
          <option value="false">{t("filters.unauthorizedOnly")}</option>
        </select>
        <input
          type="text"
          className="border rounded px-2 py-1 text-sm"
          placeholder={t("filters.objectPlaceholder")}
          value={objectFilter ?? ""}
          onChange={(e) => setObjectFilter(e.target.value || undefined)}
        />
        <input
          type="text"
          className="border rounded px-2 py-1 text-sm"
          placeholder={t("filters.actionPlaceholder")}
          value={actionFilter ?? ""}
          onChange={(e) => setActionFilter(e.target.value || undefined)}
        />
      </div>
      <PaginatedTable<AuditEntry>
        columns={columns}
        data={data?.audit as PaginatedData<AuditEntry>}
        loading={loading}
        pageSize={DEFAULT_PAGESIZE}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        initialPage={page}
        onPageChange={handlePageChange}
      />
    </div>
  )
}

export default AuditLogsList
