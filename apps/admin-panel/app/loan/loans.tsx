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

gql`
  query Loans($first: Int!, $after: String) {
    loans(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          status
          customer {
            email
          }
        }
      }
      pageInfo {
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
              <TableHead>Loan ID</TableHead>
              <TableHead>Customer Email</TableHead>
              <TableHead>Status</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data?.loans.edges.map((edge) => {
              const loan = edge?.node
              return (
                <TableRow key={loan.id}>
                  <TableCell>{loan.id}</TableCell>
                  <TableCell>{loan.customer.email}</TableCell>
                  <TableCell>{loan.status}</TableCell>
                  <TableCell>
                    <DropdownMenu>
                      <DropdownMenuTrigger>
                        <Button variant="ghost">
                          <IoEllipsisHorizontal className="w-4 h-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent className="text-sm">
                        <Link href={`/loan?loanId=${loan.id.substring(5)}`}>
                          <DropdownMenuItem>View details</DropdownMenuItem>
                        </Link>
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
