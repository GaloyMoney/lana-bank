"use client"

import { useState } from "react"
import { useRouter, useSearchParams } from "next/navigation"
import { gql } from "@apollo/client"

import Link from "next/link"

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
import { useDepositQuery, useDepositsQuery } from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"

gql`
  query Deposits($first: Int!, $after: String) {
    deposits(first: $first, after: $after) {
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
      nodes {
        customerId
        depositId
        amount
        reference
        customer {
          customerId
          email
        }
      }
    }
  }

  query Deposit($id: UUID!) {
    deposit(id: $id) {
      customerId
      depositId
      amount
      reference
      customer {
        customerId
        email
        applicantId
      }
    }
  }
`

function DepositsTable() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const searchQuery = searchParams.get("id") || ""
  const [searchInput, setSearchInput] = useState<string>(searchQuery)

  const pageSize = 100

  const {
    data: depositsData,
    error: depositsError,
    loading: depositsLoading,
    fetchMore,
  } = useDepositsQuery({
    variables: { first: pageSize },
    skip: !!searchQuery,
  })

  const {
    data: depositData,
    error: depositError,
    loading: depositLoading,
  } = useDepositQuery({
    variables: { id: searchQuery },
    skip: !searchQuery,
  })

  const handleSearch = () => {
    if (searchInput) {
      router.push(`/deposits?id=${searchInput}`)
    } else {
      router.push("/deposits")
    }
  }

  const handleClearSearch = () => {
    setSearchInput("")
    router.push("/deposits")
  }

  const handleLoadMore = () => {
    if (depositsData?.deposits.pageInfo?.hasNextPage) {
      fetchMore({
        variables: {
          after: depositsData.deposits.pageInfo.endCursor,
          first: pageSize,
        },
        updateQuery: (prev, { fetchMoreResult }) => {
          if (!fetchMoreResult) return prev
          return {
            deposits: {
              ...fetchMoreResult.deposits,
              nodes: [...prev.deposits.nodes, ...fetchMoreResult.deposits.nodes],
              pageInfo: fetchMoreResult.deposits.pageInfo,
            },
          }
        },
      })
    }
  }

  const depositsToDisplay = searchQuery
    ? depositData?.deposit
      ? [depositData.deposit]
      : []
    : depositsData?.deposits.nodes || []

  return (
    <main>
      <div className="flex justify-between items-center mb-8">
        <PageHeading className="mb-0">Deposits</PageHeading>
        <div className="flex gap-2">
          <Input
            placeholder="Find a Deposit by Deposit ID"
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
      {depositsLoading || depositLoading ? (
        <Card>
          <CardContent className="p-6">
            <div>Loading...</div>
          </CardContent>
        </Card>
      ) : depositsError || depositError ? (
        <Card>
          <CardContent className="p-6">
            <div className="text-destructive">
              {depositsError?.message || depositError?.message}
            </div>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="pt-6">
            {depositsToDisplay.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Customer Email</TableHead>
                    <TableHead>Deposit ID</TableHead>
                    <TableHead>Reference</TableHead>
                    <TableHead className="text-right">Deposit Amount</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {depositsToDisplay.map((deposit) =>
                    deposit && deposit.customer ? (
                      <TableRow key={deposit.depositId}>
                        <TableCell className="hover:underline">
                          <Link href={`/customers/${deposit.customer.customerId}`}>
                            {deposit.customer.email}
                          </Link>
                        </TableCell>
                        <TableCell>{deposit.depositId}</TableCell>
                        <TableCell>
                          {deposit.reference === deposit.depositId
                            ? "n/a"
                            : deposit.reference}
                        </TableCell>
                        <TableCell>
                          <Balance amount={deposit.amount} currency="usd" align="end" />
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
      {!searchQuery && depositsData?.deposits.pageInfo?.hasNextPage && (
        <div className="flex justify-center mt-4">
          <Button variant="ghost" onClick={handleLoadMore}>
            Load More
          </Button>
        </div>
      )}
    </main>
  )
}

export default DepositsTable
