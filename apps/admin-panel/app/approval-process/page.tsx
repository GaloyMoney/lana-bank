"use client"
import { useState } from "react"
import { gql } from "@apollo/client"
import { IoEllipsisHorizontal } from "react-icons/io5"

import { useRouter } from "next/navigation"

import { ApprovalDialog } from "./approve"
import { DenialDialog } from "./deny"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { Card, CardContent } from "@/components/primitive/card"
import { Button } from "@/components/primitive/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/primitive/dropdown-menu"
import { PageHeading } from "@/components/page-heading"
import { ApprovalProcess, useApprovalProcessesQuery } from "@/lib/graphql/generated"
import { formatDate, formatProcessType } from "@/lib/utils"

gql`
  query ApprovalProcesses($first: Int!, $after: String) {
    approvalProcesses(first: $first, after: $after) {
      pageInfo {
        hasNextPage
        endCursor
      }
      edges {
        cursor
        node {
          id
          approvalProcessId
          approvalProcessType
          createdAt
        }
      }
    }
  }
`

export default function ApprovalProcessesTable() {
  const [selectedProcessForApproval, setSelectedProcessForApproval] =
    useState<ApprovalProcess | null>(null)
  const [selectedProcessForDenial, setSelectedProcessForDenial] =
    useState<ApprovalProcess | null>(null)
  const router = useRouter()
  const { data, loading, error, fetchMore, refetch } = useApprovalProcessesQuery({
    variables: { first: 20 },
    fetchPolicy: "cache-and-network",
  })

  const handleLoadMore = () => {
    if (data?.approvalProcesses.pageInfo?.hasNextPage) {
      fetchMore({
        variables: {
          after: data.approvalProcesses.pageInfo.endCursor,
          first: 20,
        },
      })
    }
  }

  if (loading) return <div>Loading...</div>
  if (error) return <div>Error: {error.message}</div>

  return (
    <main>
      <div className="flex justify-between items-center mb-8">
        <PageHeading className="mb-0">Approval Processes</PageHeading>
      </div>

      <Card>
        <CardContent className="pt-6">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Process Type</TableHead>
                <TableHead>Created At</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data?.approvalProcesses.edges.map(({ node: process }) => (
                <TableRow
                  key={process.approvalProcessId}
                  className="cursor-pointer"
                  onClick={() => {
                    router.push(
                      processTypeLink(
                        process.approvalProcessType,
                        process.approvalProcessId,
                      ),
                    )
                  }}
                >
                  <TableCell>{formatProcessType(process.approvalProcessType)}</TableCell>
                  <TableCell>{formatDate(process.createdAt)}</TableCell>
                  <TableCell onClick={(e) => e.stopPropagation()}>
                    <DropdownMenu>
                      <DropdownMenuTrigger>
                        <Button variant="ghost">
                          <IoEllipsisHorizontal className="w-4 h-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent className="text-sm">
                        <DropdownMenuItem
                          onClick={() =>
                            setSelectedProcessForApproval(process as ApprovalProcess)
                          }
                        >
                          Approve
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onClick={() =>
                            setSelectedProcessForDenial(process as ApprovalProcess)
                          }
                        >
                          Deny
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onClick={() => {
                            router.push(
                              processTypeLink(
                                process.approvalProcessType,
                                process.approvalProcessId,
                              ),
                            )
                          }}
                        >
                          View details
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>

          {data?.approvalProcesses.pageInfo?.hasNextPage && (
            <div className="flex justify-center mt-4">
              <Button variant="ghost" onClick={handleLoadMore}>
                Load More
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {selectedProcessForApproval && (
        <ApprovalDialog
          approvalProcess={selectedProcessForApproval}
          openApprovalDialog={Boolean(selectedProcessForApproval)}
          setOpenApprovalDialog={() => {
            setSelectedProcessForApproval(null)
          }}
          refetch={refetch}
        />
      )}

      {selectedProcessForDenial && (
        <DenialDialog
          approvalProcess={selectedProcessForDenial}
          openDenialDialog={Boolean(selectedProcessForDenial)}
          setOpenDenialDialog={() => {
            setSelectedProcessForDenial(null)
          }}
          refetch={refetch}
        />
      )}
    </main>
  )
}

const processTypeLink = (processType: string, approvalProcessId: string) => {
  switch (processType) {
    case "credit-facility":
      return `/credit-facilities/${approvalProcessId}`
    case "withdraw":
      return `/withdrawals/${approvalProcessId}`
    default:
      return "/approval-process"
  }
}
