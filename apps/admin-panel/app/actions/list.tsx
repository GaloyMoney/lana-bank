"use client"

import Link from "next/link"
import { gql } from "@apollo/client"
import { HiCheckCircle } from "react-icons/hi"
import { useRouter } from "next/navigation"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/primitive/card"
import {
  ApprovalProcessStatus,
  ApprovalProcessType,
  useAllActionsQuery,
} from "@/lib/graphql/generated"
import { formatDate, formatProcessType } from "@/lib/utils"
import { CardSkeleton } from "@/components/card-skeleton"

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
              customer {
                email
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

const List: React.FC<ListProps> = ({ dashboard = false }) => {
  const router = useRouter()
  const { data, loading } = useAllActionsQuery()
  const approvalProcesses =
    data?.approvalProcesses.edges
      .filter((e) => e.node.subjectCanSubmitDecision)
      .filter((e) => e.node.status === ApprovalProcessStatus.InProgress) || []

  const quantifiedData = dashboard
    ? approvalProcesses.slice(0, NUMBER_OF_ITEMS_IN_DASHBOARD)
    : approvalProcesses

  const tableData = quantifiedData.map((e) => e.node)

  const more = tableData.length - NUMBER_OF_ITEMS_IN_DASHBOARD

  if (loading) return <CardSkeleton />

  return (
    <Card>
      <CardHeader>
        <CardTitle>Pending Actions</CardTitle>
        <CardDescription>Approvals / Rejections waiting your way</CardDescription>
      </CardHeader>

      {tableData.length > 0 ? (
        <CardContent>
          <div className="overflow-auto">
            <Table>
              {!dashboard && (
                <TableHeader>
                  <TableRow>
                    <TableHead>Customer</TableHead>
                    <TableHead>Type</TableHead>
                    <TableHead>Date</TableHead>
                    <TableHead className="w-24"></TableHead>
                  </TableRow>
                </TableHeader>
              )}
              <TableBody>
                {tableData.map((data, idx) => (
                  <TableRow key={idx}>
                    <TableCell className="font-medium">
                      {data.target.__typename === "CreditFacilityDisbursal"
                        ? ""
                        : data.target.customer.email}
                    </TableCell>
                    <TableCell>{formatProcessType(data.approvalProcessType)}</TableCell>
                    <TableCell>{formatDate(data.createdAt)}</TableCell>
                    <TableCell
                      className="text-xs font-bold cursor-pointer"
                      onClick={() => {
                        if (
                          data.approvalProcessType ===
                            ApprovalProcessType.CreditFacilityApproval &&
                          data.target.__typename === "CreditFacility"
                        )
                          router.push(
                            `/credit-facilities/${data.target.creditFacilityId}`,
                          )
                        else if (
                          data.approvalProcessType ===
                            ApprovalProcessType.WithdrawalApproval &&
                          data.target.__typename === "Withdrawal"
                        )
                          router.push(`/withdrawals/${data.target.withdrawalId}`)
                      }}
                    >
                      VIEW
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>

            {dashboard && more > 0 && (
              <div className="mt-4 flex items-center gap-2">
                <Link href="/actions" className="text-sm text-muted-foreground">
                  ...{more} more
                </Link>
              </div>
            )}
          </div>
        </CardContent>
      ) : (
        <CardContent className="flex flex-col items-start justify-center w-full gap-2 ">
          <HiCheckCircle className="text-6xl text-green-500" />
          <div className="text-sm">All Caught Up</div>
        </CardContent>
      )}
    </Card>
  )
}

export default List
