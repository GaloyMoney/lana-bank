"use client"

import { gql } from "@apollo/client"
import { IoEllipsisHorizontal } from "react-icons/io5"
import Link from "next/link"

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
import { formatCurrency, formatDate } from "@/lib/utils"

gql`
  query Loans($first: Int!, $after: String) {
    loans(first: $first, after: $after) {
      edges {
        cursor
        node {
          loanId
          status
          startDate
          customer {
            customerId
            email
          }
          balance {
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

const Loans = () => {
  const { data, loading, fetchMore } = useLoansQuery({
    variables: {
      first: 10,
    },
  })

  if (loading) {
    return <div className="mt-5">Loading...</div>
  }

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
              <TableHead>Start Date</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Customer</TableHead>
              <TableHead>Outstanding Balance</TableHead>
              <TableHead></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data?.loans.edges.map((edge) => {
              const loan = edge?.node
              return (
                <TableRow key={loan.loanId}>
                  <TableCell>{formatDate(loan.startDate)}</TableCell>
                  <TableCell>{loan.status}</TableCell>
                  <TableCell>
                    <div className="flex flex-col gap-1">
                      <div>{loan.customer.email}</div>
                      <div className="text-xs text-textColor-secondary">
                        {loan.customer.customerId}
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-col gap-1">
                      <div>
                        {formatCurrency({
                          amount: loan.balance.outstanding.usdBalance,
                          currency: "USD",
                        })}
                      </div>
                      <div className="text-xs text-textColor-secondary">
                        Interest:{" "}
                        {formatCurrency({
                          amount: loan.balance.interestIncurred.usdBalance,
                          currency: "USD",
                        })}
                      </div>
                    </div>
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
                          <Link href={`/loan?loanId=${loan.loanId}`}>
                            View Loan details
                          </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                          <Link href={`/customer/${loan.customer.customerId}`}>
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

export default Loans
