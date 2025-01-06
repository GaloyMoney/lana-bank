"use client"

import Link from "next/link"
import { gql } from "@apollo/client"
import { HiCheckCircle } from "react-icons/hi"

import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "@/ui/card"
import {
  ApprovalProcessStatus,
  ApprovalProcessType,
  useAllActionsQuery,
} from "@/lib/graphql/generated"
import { formatDate, formatProcessType } from "@/lib/utils"
import DataTable, { Column } from "@/components/data-table"
import { Skeleton } from "@/ui/skeleton"

const NUMBER_OF_ITEMS_IN_DASHBOARD = 3

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
          subjectCanSubmitDecision
          createdAt
          target {
            __typename
            ... on Withdrawal {
              withdrawalId
              account {
                customer {
                  email
                }
              }
            }
            ... on CreditFacility {
              creditFacilityId
              customer {
                email
              }
            }
            ... on CreditFacilityDisbursal {
              id
              index
              disbursalId
              creditFacility {
                customer {
                  email
                }
              }
            }
          }
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
  const { data, loading } = useAllActionsQuery({
    fetchPolicy: "cache-and-network",
  })

  const approvalProcesses =
    data?.approvalProcesses.edges
      .filter((e) => e.node.subjectCanSubmitDecision)
      .filter((e) => e.node.status === ApprovalProcessStatus.InProgress)
      .map((e) => e.node) || []

  const tableData = dashboard
    ? approvalProcesses.slice(0, NUMBER_OF_ITEMS_IN_DASHBOARD)
    : approvalProcesses

  const more = approvalProcesses.length - NUMBER_OF_ITEMS_IN_DASHBOARD

  if (loading) return <ActionListSkeleton />

  const getVisitUrl = (data: ActionNode) => {
    if (
      data.approvalProcessType === ApprovalProcessType.CreditFacilityApproval &&
      data.target.__typename === "CreditFacility"
    ) {
      return `/credit-facilities/${data.target.creditFacilityId}`
    } else if (
      data.approvalProcessType === ApprovalProcessType.WithdrawalApproval &&
      data.target.__typename === "Withdrawal"
    ) {
      return `/withdrawals/${data.target.withdrawalId}`
    } else if (
      data.approvalProcessType === ApprovalProcessType.DisbursalApproval &&
      data.target.__typename === "CreditFacilityDisbursal"
    ) {
      return `/disbursals/${data.target.disbursalId}`
    }
    return "#"
  }

  const columns: Column<ActionNode>[] = [
    {
      key: "target",
      header: "Customer",
      render: (target) => {
        switch (target.__typename) {
          case "CreditFacilityDisbursal":
            return target.creditFacility.customer.email
          case "CreditFacility":
            return target.customer.email
          case "Withdrawal":
            return target.account.customer.email
          default:
            return "Unknown"
        }
      },
    },
    {
      key: "approvalProcessType",
      header: "Type",
      render: (type) => formatProcessType(type),
    },
    {
      key: "createdAt",
      header: "Date",
      render: (date) => formatDate(date, { includeTime: false }),
    },
  ]

  return (
    <Card data-testid="dashboard-actions-list">
      <CardHeader>
        <CardTitle>Pending Actions</CardTitle>
        <CardDescription>Approvals / Rejections waiting your way</CardDescription>
      </CardHeader>

      {tableData.length > 0 ? (
        <CardContent>
          <DataTable
            data={tableData}
            columns={columns}
            navigateTo={getVisitUrl}
            className="w-full"
          />
          {dashboard && more > 0 && (
            <div className="mt-4 flex items-center gap-2">
              <Link href="/actions" className="text-sm text-muted-foreground">
                ...{more} more
              </Link>
            </div>
          )}
        </CardContent>
      ) : (
        <CardContent className="flex flex-col items-center justify-center w-full gap-2">
          <div className="border rounded-lg w-full flex flex-col items-center py-6">
            <HiCheckCircle className="text-5xl text-green-500" />
            <div className="text-sm mt-2">All Caught Up</div>
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
