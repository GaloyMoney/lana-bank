"use client"

import React from "react"
import { useTranslations } from "next-intl"
import Link from "next/link"

import { formatDate } from "@lana/web/utils"

import CardWrapper from "@/components/card-wrapper"
import DataTable, { Column } from "@/components/data-table"
import { EventPayload } from "@/components/event-payload"
import { CustomerEventHistoryQuery } from "@/lib/graphql/generated"

type EventNode = NonNullable<
  CustomerEventHistoryQuery["customerByPublicId"]
>["eventHistory"]["nodes"][number]

type EntityEventHistoryProps = {
  translationNamespace: string
  eventTranslationNamespace: string
  events: EventNode[]
  loading: boolean
}

export const EntityEventHistory: React.FC<EntityEventHistoryProps> = ({
  translationNamespace,
  eventTranslationNamespace,
  events,
  loading,
}) => {
  const t = useTranslations(translationNamespace)
  const te = useTranslations(eventTranslationNamespace)

  const translateEventType = (eventType: string): string => {
    const key = eventType.toLowerCase()
    if (te.has(key)) {
      return te(key)
    }
    return eventType
  }

  const columns: Column<EventNode>[] = [
    {
      key: "eventType",
      header: t("columns.event"),
      render: (eventType: string) => translateEventType(eventType),
    },
    {
      key: "payload",
      header: t("columns.details"),
      render: (payload: Record<string, unknown>) => <EventPayload payload={payload} />,
    },
    {
      key: "subject",
      header: t("columns.subject"),
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
      header: t("columns.auditEntryId"),
      render: (auditEntryId) => (
        <span className="text-muted-foreground text-xs">{auditEntryId ?? "-"}</span>
      ),
    },
    {
      key: "recordedAt",
      header: t("columns.recordedAt"),
      render: (recordedAt: string) => formatDate(recordedAt),
    },
  ]

  return (
    <CardWrapper title={t("title")} description={t("description")}>
      <DataTable
        data={events}
        columns={columns}
        loading={loading}
        emptyMessage={t("emptyMessage")}
      />
    </CardWrapper>
  )
}
