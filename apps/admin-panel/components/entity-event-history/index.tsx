"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import CardWrapper from "@/components/card-wrapper"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { EventPayload } from "@/components/event-payload"
import { AuditUser } from "@/components/audit-user"
import { EventTimelineEntry } from "@/lib/graphql/generated"

type EntityEventHistoryProps = {
  title: string
  description: string
  emptyMessage: string
  translationNamespace: string
  data?: PaginatedData<EventTimelineEntry>
  loading: boolean
  fetchMore: (cursor: string) => Promise<unknown>
}

export const EntityEventHistory: React.FC<EntityEventHistoryProps> = ({
  title,
  description,
  emptyMessage,
  translationNamespace,
  data,
  loading,
  fetchMore,
}) => {
  const te = useTranslations(translationNamespace)
  const tc = useTranslations("Common")
  const t = useTranslations("EntityEvents")

  const translateEventType = (eventType: string): string => {
    const key = eventType.toLowerCase()
    if (te.has(key)) {
      return te(key)
    }
    return eventType
  }

  const columns: Column<EventTimelineEntry>[] = [
    {
      key: "eventType",
      label: t("columns.event"),
      render: (eventType: string) => translateEventType(eventType),
    },
    {
      key: "payload",
      label: t("columns.details"),
      render: (payload: Record<string, unknown>) => <EventPayload payload={payload} />,
    },
    {
      key: "userId",
      label: t("columns.userId"),
      render: (userId) =>
        userId ? (
          <AuditUser subjectId={userId} />
        ) : (
          <span className="text-muted-foreground text-xs">{tc("system")}</span>
        ),
    },
    {
      key: "auditEntryId",
      label: t("columns.auditEntryId"),
      render: (auditEntryId) => (
        <span className="text-muted-foreground text-xs">{auditEntryId ?? "-"}</span>
      ),
    },
    {
      key: "recordedAt",
      label: t("columns.recordedAt"),
      render: (recordedAt: string) => formatDate(recordedAt),
    },
  ]

  return (
    <CardWrapper title={title} description={description}>
      <PaginatedTable<EventTimelineEntry>
        columns={columns}
        data={data}
        loading={loading}
        pageSize={DEFAULT_PAGESIZE}
        fetchMore={fetchMore}
        noDataText={emptyMessage}
      />
    </CardWrapper>
  )
}
