"use client"

import React from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import Link from "next/link"

import { formatDate } from "@lana/web/utils"

import CardWrapper from "@/components/card-wrapper"
import DataTable, { Column } from "@/components/data-table"
import { EventPayload } from "@/components/event-payload"
import { useDepositEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query DepositEventHistory($id: PublicId!, $first: Int!, $after: String) {
    depositByPublicId(id: $id) {
      id
      eventHistory(first: $first, after: $after) {
        edges {
          cursor
          node {
            eventType
            recordedAt
            sequence
            auditEntryId
            subject {
              ... on User { userId, email }
              ... on System { actor }
            }
            payload
          }
        }
        nodes {
          eventType
          recordedAt
          sequence
          auditEntryId
          subject {
            ... on User { userId, email }
            ... on System { actor }
          }
          payload
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
  }
`

type DepositEventHistoryProps = {
  depositPublicId: string
}

export const DepositEventHistory: React.FC<DepositEventHistoryProps> = ({
  depositPublicId,
}) => {
  const t = useTranslations("Deposits.DepositDetails.eventHistory")
  const te = useTranslations("EntityEvents.deposit")

  const { data, loading } = useDepositEventHistoryQuery({
    variables: { id: depositPublicId, first: 100 },
  })

  const events = data?.depositByPublicId?.eventHistory.nodes ?? []
  type EventNode = (typeof events)[number]

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
