"use client"

import { gql } from "@apollo/client"
import { IoEllipsisHorizontal } from "react-icons/io5"
import Link from "next/link"

import { LoanStatusBadge } from "./status-badge"

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/primitive/dropdown-menu"
import { Button } from "@/components/primitive/button"
import { useLoansQuery } from "@/lib/graphql/generated"
import { Card, CardContent } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { formatDate } from "@/lib/utils"
import Balance from "@/components/balance/balance"

gql`
  query Loans($first: Int!, $after: String) {
    loans(first: $first, after: $after) {
      edges {
        cursor
        node {
          loanId
          status
          createdAt
          customer {
            customerId
            email
          }
          principal
          currentCvl @client
          balance {
            collateral {
              btcBalance
            }
            outstanding {
              usdBalance
            }
            interestIncurred {
              usdBalance
            }
          }
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }
`

const LoansTable = () => {
  const { data, loading, error, fetchMore } = useLoansQuery({
    variables: {
      first: 10,
    },
    fetchPolicy: "cache-and-network",
  })

  if (loading) {
    return <div className="mt-5">Loading...</div>
  }

  if (error) return <div className="text-destructive">{error.message}</div>

  if (data?.loans.edges.length === 0) {
    return (
      <Card className="mt-5">
        <CardContent className="pt-6">No loans found</CardContent>
      </Card>
    )
  }

  return (
    <Card className="mt-5">
      <CardContent className="pt-6">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Created At</TableHead>
              <TableHead>Customer</TableHead>
              <TableHead>Current CVL</TableHead>
              <TableHead>Outstanding Balance</TableHead>
              <TableHead>Status</TableHead>
              <TableHead></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data?.loans.edges.map((edge) => {
              const loan = edge?.node
              return (
                <TableRow key={loan.loanId}>
                  <TableCell>{formatDate(loan.createdAt)}</TableCell>
                  <TableCell className="hover:underline">
                    <Link href={`/customers/${loan.customer.customerId}`}>
                      {loan.customer.email}
                    </Link>
                  </TableCell>
                  <TableCell>
                    <span className="font-mono">{loan.currentCvl}</span>
                    {" %"}
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-col gap-1">
                      <Balance
                        amount={loan.balance.outstanding.usdBalance}
                        currency="usd"
                      />
                      <div className="text-xs text-textColor-secondary flex gap-1">
                        Interest:{" "}
                        <Balance
                          amount={loan.balance.interestIncurred.usdBalance}
                          currency="usd"
                        />
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <LoanStatusBadge status={loan.status} className="p-1 px-4" />
                  </TableCell>
                  <TableCell>
                    <DropdownMenu>
                      <DropdownMenuTrigger>
                        <Button variant="ghost">
                          <IoEllipsisHorizontal className="w-4 h-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent className="text-sm">
                        <DropdownMenuItem>
                          <Link href={`/loans/${loan.loanId}`}>View Loan details</Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                          <Link href={`/customers/${loan.customer.customerId}`}>
                            View Customer details
                          </Link>
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              )
            })}
            {data?.loans.pageInfo.hasNextPage && (
              <TableRow
                className="cursor-pointer"
                onClick={() =>
                  fetchMore({
                    variables: {
                      after: data.loans.edges[data.loans.edges.length - 1].cursor,
                    },
                  })
                }
              >
                <TableCell>
                  <div className="font-thin italic">show more...</div>
                </TableCell>
                <TableCell />
                <TableCell />
                <TableCell />
              </TableRow>
            )}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}

export default LoansTable
