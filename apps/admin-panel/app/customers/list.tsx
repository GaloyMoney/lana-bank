"use client"

import { useState } from "react"
import Link from "next/link"
import { IoEllipsisHorizontal } from "react-icons/io5"

import {
  CustomersQuery,
  useCustomersQuery,
  useGetCustomerByCustomerIdQuery,
  useGetCustomerByCustomerEmailQuery,
} from "@/lib/graphql/generated"
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
import Balance from "@/components/balance/balance"

import { CreateLoanDialog } from "@/app/loans/create"
import { WithdrawalInitiateDialog } from "@/app/withdrawals/initiate"
import { RecordDepositDialog } from "@/app/deposits/record"

type CustomerType = NonNullable<CustomersQuery["customers"]["nodes"][number]>

function CustomerTable({
  searchValue,
  searchType,
  renderCreateCustomerDialog,
}: {
  searchValue?: string
  searchType?: "customerId" | "email" | "unknown"
  renderCreateCustomerDialog: (refetch: () => void) => React.ReactNode
}) {
  const [openWithdrawalInitiateDialog, setOpenWithdrawalInitiateDialog] = useState<
    string | null
  >(null)
  const [openRecordDepositDialog, setOpenRecordDepositDialog] = useState<string | null>(
    null,
  )

  const pageSize = 100

  const {
    data: customersData,
    error: customersError,
    loading: customersLoading,
    refetch: refetchCustomers,
    fetchMore,
  } = useCustomersQuery({
    variables: { first: pageSize },
    skip: !!searchValue,
    fetchPolicy: "cache-and-network",
  })

  const {
    data: customerByIdData,
    error: customerByIdError,
    loading: customerByIdLoading,
  } = useGetCustomerByCustomerIdQuery({
    variables: { id: searchValue || "" },
    skip: !searchValue || searchType !== "customerId",
  })

  const {
    data: customerByEmailData,
    error: customerByEmailError,
    loading: customerByEmailLoading,
  } = useGetCustomerByCustomerEmailQuery({
    variables: { email: searchValue || "" },
    skip: !searchValue || searchType !== "email",
  })

  let customerDetails: CustomerType[] | null = null
  let error: string | null = null
  const loading = customersLoading || customerByIdLoading || customerByEmailLoading

  if (searchType === "customerId" && customerByIdData?.customer) {
    customerDetails = [customerByIdData.customer]
    error = customerByIdError?.message || null
  } else if (searchType === "email" && customerByEmailData?.customerByEmail) {
    customerDetails = [customerByEmailData.customerByEmail]
    error = customerByEmailError?.message || null
  } else if (!searchValue) {
    customerDetails = customersData?.customers.nodes || null
    error = customersError?.message || null
  } else if (searchType === "unknown") {
    error = "Invalid search input. Please enter a valid customer ID or email."
  }

  const pageInfo = customersData?.customers.pageInfo

  const handleLoadMore = () => {
    if (pageInfo?.hasNextPage) {
      fetchMore({
        variables: {
          after: pageInfo.endCursor,
          first: pageSize,
        },
        updateQuery: (prev, { fetchMoreResult }) => {
          if (!fetchMoreResult) return prev
          return {
            customers: {
              ...fetchMoreResult.customers,
              nodes: [...prev.customers.nodes, ...fetchMoreResult.customers.nodes],
            },
          }
        },
      })
    }
  }

  return (
    <>
      {renderCreateCustomerDialog(refetchCustomers)}
      {loading ? (
        <Card>
          <CardContent className="p-6">
            <div>Loading...</div>
          </CardContent>
        </Card>
      ) : error ? (
        <Card>
          <CardContent className="p-6">
            <div className="text-destructive">{error}</div>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="pt-6">
            {customerDetails && customerDetails.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Customer</TableHead>
                    <TableHead>USD Balance (Settled)</TableHead>
                    <TableHead>USD Balance (Withdrawals)</TableHead>
                    <TableHead></TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {customerDetails.map((customer) => (
                    <TableRow key={customer.customerId}>
                      <TableCell className="hover:underline">
                        <Link href={`/customers/${customer.customerId}`}>
                          {customer.email}
                        </Link>
                      </TableCell>
                      <TableCell>
                        <Balance
                          amount={customer.balance.checking.settled}
                          currency="usd"
                        />
                      </TableCell>
                      <TableCell>
                        <Balance
                          amount={customer.balance.checking.pending}
                          currency="usd"
                        />
                      </TableCell>
                      <TableCell>
                        <DropdownMenu>
                          <DropdownMenuTrigger>
                            <Button variant="ghost">
                              <IoEllipsisHorizontal className="w-4 h-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent className="text-sm">
                            <Link href={`/customers/${customer.customerId}`}>
                              <DropdownMenuItem>View details</DropdownMenuItem>
                            </Link>
                            <DropdownMenuItem onClick={(e) => e.preventDefault()}>
                              <CreateLoanDialog customerId={customer.customerId}>
                                <span>Create Loan</span>
                              </CreateLoanDialog>
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() =>
                                setOpenRecordDepositDialog(customer.customerId)
                              }
                            >
                              Record Deposit
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() =>
                                setOpenWithdrawalInitiateDialog(customer.customerId)
                              }
                            >
                              Record Withdrawal
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div>No data available</div>
            )}
          </CardContent>
        </Card>
      )}
      {!searchValue && pageInfo?.hasNextPage && (
        <div className="flex justify-center mt-4">
          <Button variant="ghost" onClick={handleLoadMore}>
            Load More
          </Button>
        </div>
      )}
      {openWithdrawalInitiateDialog && (
        <WithdrawalInitiateDialog
          customerId={openWithdrawalInitiateDialog}
          openWithdrawalInitiateDialog={Boolean(openWithdrawalInitiateDialog)}
          setOpenWithdrawalInitiateDialog={() => setOpenWithdrawalInitiateDialog(null)}
        />
      )}
      {openRecordDepositDialog && (
        <RecordDepositDialog
          customerId={openRecordDepositDialog}
          openRecordDepositDialog={Boolean(openRecordDepositDialog)}
          setOpenRecordDepositDialog={() => setOpenRecordDepositDialog(null)}
        />
      )}
    </>
  )
}

export default CustomerTable
