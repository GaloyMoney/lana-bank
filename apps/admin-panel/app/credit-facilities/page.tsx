"use client"

import React, { useState } from "react"
import { useRouter } from "next/navigation"
import { gql } from "@apollo/client"
import { IoEllipsisHorizontal } from "react-icons/io5"

import Link from "next/link"

import { Button } from "@/components/primitive/button"
import { Input } from "@/components/primitive/input"
import { PageHeading } from "@/components/page-heading"
import { Card, CardContent } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/primitive/dropdown-menu"
import Balance from "@/components/balance/balance"
import { useCreditFacilitiesQuery } from "@/lib/graphql/generated"

gql`
  query CreditFacilities($first: Int!, $after: String) {
    creditFacilities(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          creditFacilityId
          collateralizationState
          balance {
            outstanding {
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

const CreditFacilitiesTable = () => {
  const { data, loading, error, fetchMore } = useCreditFacilitiesQuery({
    variables: {
      first: 10,
    },
    fetchPolicy: "cache-and-network",
  })

  if (loading) return <div className="mt-5">Loading...</div>
  if (error) return <div className="text-destructive">{error.message}</div>

  if (data?.creditFacilities.edges.length === 0) {
    return (
      <Card className="mt-5">
        <CardContent className="pt-6">No credit facilities found</CardContent>
      </Card>
    )
  }

  return (
    <Card className="mt-5">
      <CardContent className="pt-6">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Credit Facility ID</TableHead>
              <TableHead>Outstanding Balance</TableHead>
              <TableHead>Collateralization State</TableHead>
              <TableHead></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data?.creditFacilities.edges.map((edge) => {
              const facility = edge?.node
              return (
                <TableRow key={facility.creditFacilityId}>
                  <Link
                    href={`/credit-facilities/${facility.creditFacilityId}`}
                    className="flex items-center hover:underline"
                  >
                    <TableCell>{facility.creditFacilityId}</TableCell>
                  </Link>
                  <TableCell>
                    <Balance
                      amount={facility.balance.outstanding.usdBalance}
                      currency="usd"
                    />
                  </TableCell>
                  <TableCell>{facility.collateralizationState}</TableCell>
                  <TableCell>
                    <DropdownMenu>
                      <DropdownMenuTrigger>
                        <Button variant="ghost">
                          <IoEllipsisHorizontal className="w-4 h-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent className="text-sm">
                        <DropdownMenuItem>
                          <Link href={`/credit-facilities/${facility.creditFacilityId}`}>
                            View Credit Facility details
                          </Link>
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              )
            })}
            {data?.creditFacilities.pageInfo.hasNextPage && (
              <TableRow
                className="cursor-pointer"
                onClick={() =>
                  fetchMore({
                    variables: {
                      after: data.creditFacilities.pageInfo.endCursor,
                    },
                  })
                }
              >
                <TableCell colSpan={4}>
                  <div className="font-thin italic">show more...</div>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}

const CreditFacilitiesPage = () => {
  const router = useRouter()
  const [inputCreditFacilityId, setInputCreditFacilityId] = useState("")

  const handleSearch = () => {
    router.push(`/credit-facilities/${inputCreditFacilityId}`)
  }

  return (
    <main>
      <div className="flex justify-between items-center mb-8">
        <PageHeading className="mb-0">Credit Facilities</PageHeading>
        <div className="flex gap-2">
          <Input
            onChange={(e) => setInputCreditFacilityId(e.target.value)}
            placeholder="Find a credit facility by ID"
            id="creditFacilityId"
            name="creditFacilityId"
            value={inputCreditFacilityId}
            className="w-80"
          />
          <Button onClick={handleSearch} variant="primary">
            Search
          </Button>
        </div>
      </div>

      <CreditFacilitiesTable />
    </main>
  )
}

export default CreditFacilitiesPage
