"use client"

import { useTranslations } from "next-intl"
import Link from "next/link"
import { gql } from "@apollo/client"
import { HiCheckCircle } from "react-icons/hi"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import { Skeleton } from "@lana/web/ui/skeleton"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import {
  ApprovalProcessStatus,
  useAllActionsQuery,
} from "@/lib/graphql/generated"
import { formatProcessType } from "@/lib/utils"
import DataTable, { Column } from "@/components/data-table"

gql`
  query AllActions {
    approvalProcesses(first: 1000000) {
      pageInfo {
        hasNextPage
        hasPreviousPage
      }
      edges {
        node {
          id
          approvalProcessType
          status
          userCanSubmitDecision
          createdAt
        }
        cursor
      }
    }
  }
`

type ListProps = {
  dashboard?: boolean
}

type ActionNode = NonNullable<
  NonNullable<
    NonNullable<
      ReturnType<typeof useAllActionsQuery>["data"]
    >["approvalProcesses"]["edges"][number]
  >["node"]
>

const List: React.FC<ListProps> = ({ dashboard = false }) => {
  const t = useTranslations("Actions.table")
  const { data, loading } = useAllActionsQuery({
    fetchPolicy: "cache-and-network",
  })

  const approvalProcesses =
    data?.approvalProcesses.edges
      .filter((e) => e.node.userCanSubmitDecision)
      .filter((e) => e.node.status === ApprovalProcessStatus.InProgress)
      .map((e) => e.node) || []

  const tableData = dashboard ? approvalProcesses.slice(0, 3) : approvalProcesses

  const more = approvalProcesses.length - 3

  if (loading && !data) return <ActionListSkeleton />

  const columns: Column<ActionNode>[] = [
    {
      key: "approvalProcessType",
      header: t("headers.type"),
      render: (type) => formatProcessType(type),
    },
    {
      key: "createdAt",
      header: t("headers.date"),
      render: (date) => <DateWithTooltip value={date} />,
    },
  ]

  return (
    <Card data-testid="dashboard-actions-list">
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      {tableData.length > 0 ? (
        <CardContent>
          <DataTable
            data={tableData}
            columns={columns}
            className="w-full"
          />
          {dashboard && more > 0 && (
            <div className="mt-4 flex items-center gap-2">
              <Link href="/actions" className="text-sm text-muted-foreground">
                {t("more", { count: more })}
              </Link>
            </div>
          )}
        </CardContent>
      ) : (
        <CardContent className="flex flex-col items-center justify-center w-full gap-2">
          <div className="border rounded-lg w-full flex flex-col items-center py-6">
            <HiCheckCircle className="text-5xl text-green-500" />
            <div className="text-sm mt-2">{t("allCaughtUp")}</div>
          </div>
        </CardContent>
      )}
    </Card>
  )
}

export default List

const ActionListSkeleton = () => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>
          <Skeleton className="h-8 w-32" />
        </CardTitle>
        <CardDescription>
          <Skeleton className="h-4 w-64" />
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Skeleton className="h-[115px] w-full" />
      </CardContent>
    </Card>
  )
}
