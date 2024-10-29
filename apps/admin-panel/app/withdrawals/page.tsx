"use client"
import { useState } from "react"
import { useRouter, useSearchParams } from "next/navigation"
import Link from "next/link"
import { gql } from "@apollo/client"

import WithdrawalDropdown from "./drop-down"
import { WithdrawalStatusBadge } from "./status-badge"
import { WithdrawalConfirmDialog } from "./confirm"
import { WithdrawalCancelDialog } from "./cancel"

import { useWithdrawalQuery, useWithdrawalsQuery } from "@/lib/graphql/generated"

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
import { Input } from "@/components/primitive/input"
import { PageHeading } from "@/components/page-heading"
import Balance from "@/components/balance/balance"

gql`
  query Withdrawals($first: Int!, $after: String) {
    withdrawals(first: $first, after: $after) {
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
      nodes {
        customerId
        withdrawalId
        amount
        status
        reference
        userCanConfirm
        userCanCancel
        customer {
          customerId
          email
        }
      }
    }
  }

  query Withdrawal($id: UUID!) {
    withdrawal(id: $id) {
      customerId
      withdrawalId
      amount
      status
      reference
      customer {
        customerId
        email
        applicantId
      }
    }
  }
`

function WithdrawalsTable() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const searchQuery = searchParams.get("id") || ""
  const [searchInput, setSearchInput] = useState<string>(searchQuery)
  const [openWithdrawalCancelDialog, setOpenWithdrawalCancelDialog] =
    useState<WithdrawalWithCustomer | null>(null)
  const [openWithdrawalConfirmDialog, setOpenWithdrawalConfirmDialog] =
    useState<WithdrawalWithCustomer | null>(null)

  const pageSize = 100

  const {
    data: withdrawalsData,
    error: withdrawalsError,
    loading: withdrawalsLoading,
    fetchMore,
    refetch: refetchWithdrawals,
  } = useWithdrawalsQuery({
    variables: { first: pageSize },
    skip: !!searchQuery,
    fetchPolicy: "cache-and-network",
  })

  const {
    data: withdrawalData,
    error: withdrawalError,
    loading: withdrawalLoading,
    refetch: refetchWithdrawal,
  } = useWithdrawalQuery({
    variables: { id: searchQuery },
    skip: !searchQuery,
  })

  const handleSearch = () => {
    if (searchInput) {
      router.push(`/withdrawals?id=${searchInput}`)
    } else {
      router.push("/withdrawals")
    }
  }

  const handleClearSearch = () => {
    setSearchInput("")
    router.push("/withdrawals")
  }

  const handleLoadMore = () => {
    if (withdrawalsData?.withdrawals.pageInfo?.hasNextPage) {
      fetchMore({
        variables: {
          after: withdrawalsData.withdrawals.pageInfo.endCursor,
          first: pageSize,
        },
        updateQuery: (prev, { fetchMoreResult }) => {
          if (!fetchMoreResult) return prev
          return {
            withdrawals: {
              ...fetchMoreResult.withdrawals,
              nodes: [...prev.withdrawals.nodes, ...fetchMoreResult.withdrawals.nodes],
              pageInfo: fetchMoreResult.withdrawals.pageInfo,
            },
          }
        },
      })
    }
  }

  const withdrawalsToDisplay = searchQuery
    ? withdrawalData?.withdrawal
      ? [withdrawalData.withdrawal]
      : []
    : withdrawalsData?.withdrawals.nodes || []

  return (
    <main>
      <div className="flex justify-between items-center mb-8">
        <PageHeading className="mb-0">Withdrawals</PageHeading>
        <div className="flex gap-2">
          <Input
            placeholder="Find a Withdrawal by Withdrawal ID"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            className="w-80"
          />
          <Button onClick={handleSearch}>Search</Button>
          {searchQuery && (
            <Button variant="secondary" onClick={handleClearSearch}>
              X Clear
            </Button>
          )}
        </div>
      </div>
      {withdrawalsLoading || withdrawalLoading ? (
        <Card>
          <CardContent className="p-6">
            <div>Loading...</div>
          </CardContent>
        </Card>
      ) : withdrawalsError || withdrawalError ? (
        <Card>
          <CardContent className="p-6">
            <div className="text-destructive">
              {withdrawalsError?.message || withdrawalError?.message}
            </div>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="pt-6">
            {withdrawalsToDisplay.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Customer Email</TableHead>
                    <TableHead>Withdrawal ID</TableHead>
                    <TableHead>Withdrawal Amount</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead></TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {withdrawalsToDisplay.map((withdrawal) =>
                    withdrawal && withdrawal.customer ? (
                      <TableRow
                        className="cursor-pointer"
                        onClick={() => {
                          router.push(`/withdrawals/${withdrawal.withdrawalId}`)
                        }}
                        key={withdrawal.withdrawalId}
                      >
                        <TableCell className="hover:underline">
                          <Link href={`/customers/${withdrawal.customer.customerId}`}>
                            {withdrawal.customer.email}
                          </Link>
                        </TableCell>
                        <TableCell>{withdrawal.withdrawalId}</TableCell>
                        <TableCell>
                          <Balance amount={withdrawal.amount} currency="usd" />
                        </TableCell>
                        <TableCell>
                          <WithdrawalStatusBadge status={withdrawal.status} />
                        </TableCell>

                        <TableCell>
                          <WithdrawalDropdown
                            withdrawal={withdrawal}
                            onConfirm={() => setOpenWithdrawalConfirmDialog(withdrawal)}
                            onCancel={() => setOpenWithdrawalCancelDialog(withdrawal)}
                          />
                        </TableCell>
                      </TableRow>
                    ) : null,
                  )}
                </TableBody>
              </Table>
            ) : (
              <div>No data available</div>
            )}
          </CardContent>
        </Card>
      )}
      {!searchQuery && withdrawalsData?.withdrawals.pageInfo?.hasNextPage && (
        <div className="flex justify-center mt-4">
          <Button variant="ghost" onClick={handleLoadMore}>
            Load More
          </Button>
        </div>
      )}
      {openWithdrawalConfirmDialog && (
        <WithdrawalConfirmDialog
          refetch={refetchWithdrawals || refetchWithdrawal}
          withdrawalData={openWithdrawalConfirmDialog}
          openWithdrawalConfirmDialog={Boolean(openWithdrawalConfirmDialog)}
          setOpenWithdrawalConfirmDialog={() => setOpenWithdrawalConfirmDialog(null)}
        />
      )}
      {openWithdrawalCancelDialog && (
        <WithdrawalCancelDialog
          refetch={refetchWithdrawals || refetchWithdrawal}
          withdrawalData={openWithdrawalCancelDialog}
          openWithdrawalCancelDialog={Boolean(openWithdrawalCancelDialog)}
          setOpenWithdrawalCancelDialog={() => setOpenWithdrawalCancelDialog(null)}
        />
      )}
    </main>
  )
}

export default WithdrawalsTable
