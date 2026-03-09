"use client"

import React from "react"
import { useTranslations } from "next-intl"
import Link from "next/link"

import { formatDate } from "@lana/web/utils"

import CardWrapper from "@/components/card-wrapper"
import PaginatedTable, {
  Column,
  PaginatedData,
  DEFAULT_PAGESIZE,
} from "@/components/paginated-table"
import { EventPayload } from "@/components/event-payload"
import { CustomerEventHistoryQuery } from "@/lib/graphql/generated"

type EventNode = NonNullable<
  CustomerEventHistoryQuery["customerByPublicId"]
>["eventHistory"]["edges"][number]["node"]

type EntityEventHistoryProps = {
  translationNamespace: string
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  data?: any
  loading: boolean
  fetchMore: (cursor: string) => Promise<unknown>
}

const snakeToSentenceCase = (s: string): string =>
  s
    .split("_")
    .map((word, i) => (i === 0 ? word.charAt(0).toUpperCase() + word.slice(1) : word))
    .join(" ")

export const EntityEventHistory: React.FC<EntityEventHistoryProps> = ({
  translationNamespace,
  data,
  loading,
  fetchMore,
}) => {
  const t = useTranslations(translationNamespace)

  const columns: Column<EventNode>[] = [
    {
      key: "eventType",
      label: t("columns.event"),
      render: (eventType: string) => snakeToSentenceCase(eventType),
    },
    {
      key: "payload",
      label: t("columns.details"),
      render: (payload: Record<string, unknown>) => <EventPayload payload={payload} />,
    },
    {
      key: "subject",
      label: t("columns.subject"),
      render: (subject) => {
        if (!subject) return <span className="text-muted-foreground text-xs">-</span>
        if (subject.__typename === "User") {
          return (
            <Link
              href={`/users/${subject.userId}`}
              className="text-primary underline underline-offset-4 hover:text-primary/80 text-xs"
            >
              {subject.email}
            </Link>
          )
        }
        if (subject.__typename === "System") {
          return <span className="text-xs">system ({subject.actor})</span>
        }
        return <span className="text-muted-foreground text-xs">-</span>
      },
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
    <CardWrapper title={t("title")} description={t("description")}>
      <PaginatedTable
        data={data as PaginatedData<EventNode>}
        columns={columns}
        loading={loading}
        pageSize={DEFAULT_PAGESIZE}
        fetchMore={fetchMore}
        noDataText={t("emptyMessage")}
        cellClassName="align-top"
      />
    </CardWrapper>
  )
}

export { DEFAULT_PAGESIZE }
