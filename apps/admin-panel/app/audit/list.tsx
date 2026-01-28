"use client"
import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"
import { Input } from "@lana/web/ui/input"

import {
  AuditEntry,
  AuditSubject,
  useAuditLogsQuery,
  useAuditSubjectsQuery,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

const formatSubject = (subject: AuditSubject): string => {
  switch (subject.__typename) {
    case "User":
      return subject.email
    case "System":
      return subject.actor
    default:
      return "Unknown"
  }
}

gql`
  query AuditLogs($first: Int!, $after: String, $subject: AuditSubjectId, $authorized: Boolean, $object: String, $action: String) {
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
              actor
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

const AuditLogsList = () => {
  const t = useTranslations("AuditLogs.table")

  const [subjectFilter, setSubjectFilter] = useState<string | undefined>(undefined)
  const [authorizedFilter, setAuthorizedFilter] = useState<boolean | undefined>(
    undefined,
  )
  const [objectFilter, setObjectFilter] = useState<string | undefined>(undefined)
  const [actionFilter, setActionFilter] = useState<string | undefined>(undefined)

  const { data, loading, error, fetchMore } = useAuditLogsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
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
      render: (subject) => <div>{formatSubject(subject)}</div>,
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
        <Select
          value={subjectFilter ?? "all"}
          onValueChange={(val) => setSubjectFilter(val === "all" ? undefined : val)}
        >
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder={t("filters.allSubjects")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("filters.allSubjects")}</SelectItem>
            {subjectsData?.auditSubjects.map((s) => (
              <SelectItem key={s} value={s}>
                {s}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select
          value={authorizedFilter === undefined ? "all" : String(authorizedFilter)}
          onValueChange={(val) =>
            setAuthorizedFilter(val === "all" ? undefined : val === "true")
          }
        >
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder={t("filters.allAuthorized")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("filters.allAuthorized")}</SelectItem>
            <SelectItem value="true">{t("filters.authorizedOnly")}</SelectItem>
            <SelectItem value="false">{t("filters.unauthorizedOnly")}</SelectItem>
          </SelectContent>
        </Select>
        <Input
          placeholder={t("filters.objectPlaceholder")}
          value={objectFilter ?? ""}
          onChange={(e) => setObjectFilter(e.target.value || undefined)}
        />
        <Input
          placeholder={t("filters.actionPlaceholder")}
          value={actionFilter ?? ""}
          onChange={(e) => setActionFilter(e.target.value || undefined)}
        />
      </div>
      <PaginatedTable<AuditEntry>
        key={`${subjectFilter}-${authorizedFilter}-${objectFilter}-${actionFilter}`}
        columns={columns}
        data={data?.audit as PaginatedData<AuditEntry>}
        loading={loading}
        pageSize={DEFAULT_PAGESIZE}
        fetchMore={async (cursor) =>
          fetchMore({
            variables: {
              after: cursor,
              subject: subjectFilter ?? null,
              authorized: authorizedFilter ?? null,
              object: objectFilter ?? null,
              action: actionFilter ?? null,
            },
          })
        }
      />
    </div>
  )
}

export default AuditLogsList
