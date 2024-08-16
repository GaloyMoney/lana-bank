"use client"

import { IoEllipsisHorizontal } from "react-icons/io5"
import { IoMdArrowDropdown, IoMdArrowDropup } from "react-icons/io"
import Link from "next/link"
import { useState } from "react"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import {
  GetLoansForCustomerQuery,
  Loan,
  LoanStatus,
  useGetLoansForCustomerQuery,
} from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import { CreateLoanDialog } from "@/components/loan/create-loan-dialog"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/primitive/dropdown-menu"
import { LoanPartialPaymentDialog } from "@/components/loan/loan-partial-payment"
import { LoanApproveDialog } from "@/components/loan/approve-loan"
import { LoanBadge } from "@/components/loan/loan-badge"
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/primitive/collapsible"
import { DetailItem } from "@/components/details"
import { formatInterval, formatPeriod } from "@/lib/terms/utils"
import Balance from "@/components/balance/balance"

type LoanRowProps = {
  loanId: string
  balance: {
    collateral: {
      btcBalance: number
    }
    interestIncurred: {
      usdBalance: number
    }
    outstanding: {
      usdBalance: number
    }
  }
  status: LoanStatus
  loanTerms: Loan["loanTerms"]
}

export const CustomerLoansTable = ({ customerId }: { customerId: string }) => {
  const {
    loading,
    error,
    data: customerLoans,
    refetch,
  } = useGetLoansForCustomerQuery({
    variables: {
      id: customerId,
    },
  })

  return (
    <Card className="mt-4">
      {loading ? (
        <CardContent className="p-6">Loading...</CardContent>
      ) : error ? (
        <CardContent className="p-6 text-destructive">{error.message}</CardContent>
      ) : (
        <>
          <CardHeader className="flex flex-row justify-between items-center pb-0">
            <div className="flex flex-col space-y-1.5">
              <CardTitle>Loans</CardTitle>
              <CardDescription>Loan Details for Customer</CardDescription>
            </div>
            <CreateLoanDialog refetch={refetch} customerId={customerId}>
              <Button>New Loan</Button>
            </CreateLoanDialog>
          </CardHeader>
          {!customerLoans ||
          !customerLoans.customer?.loans ||
          customerLoans.customer?.loans.length === 0 ? (
            <CardContent className="p-6">No loans found for this customer</CardContent>
          ) : (
            <CardContent>
              <LoanCountDetails customerLoans={customerLoans} />
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableCell>Loan ID</TableCell>
                    <TableCell>Collateral (BTC)</TableCell>
                    <TableCell>Interest Incurred (USD)</TableCell>
                    <TableCell>Outstanding (USD)</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell>Actions</TableCell>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {customerLoans.customer.loans.map((loan) => (
                    <LoanRow key={loan.loanId} loan={loan} refetch={refetch} />
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          )}
        </>
      )}
    </Card>
  )
}

const LoanCountCard = ({ title, count }: { title: string; count: number }) => (
  <Card variant="secondary" className="w-1/3">
    <CardHeader>
      <CardDescription>{title}</CardDescription>
      <CardTitle className="text-4xl">{count}</CardTitle>
    </CardHeader>
  </Card>
)

const LoanCountDetails = ({
  customerLoans,
}: {
  customerLoans: GetLoansForCustomerQuery
}) => {
  const initialCounts = Object.values(LoanStatus).reduce(
    (acc, status) => {
      acc[status] = 0
      return acc
    },
    {} as Record<LoanStatus, number>,
  )

  const loanCounts =
    customerLoans?.customer?.loans?.reduce((counts, loan) => {
      counts[loan.status] += 1
      return counts
    }, initialCounts) || initialCounts

  return (
    <div className="flex w-full gap-4 mt-4 mb-8">
      {Object.entries(LoanStatus).map(([key, status]) => (
        <LoanCountCard key={status} title={key} count={loanCounts[status]} />
      ))}
    </div>
  )
}

const LoanRow = ({ loan, refetch }: { loan: LoanRowProps; refetch: () => void }) => {
  const [isOpen, setIsOpen] = useState(false)

  return (
    <Collapsible asChild>
      <>
        <CollapsibleTrigger asChild>
          <TableRow onClick={() => setIsOpen(!isOpen)}>
            <TableCell>
              <div className="flex items-center gap-2">
                {loan.loanId}
                <Button
                  variant="ghost"
                  className="p-2"
                  onClick={() => setIsOpen(!isOpen)}
                >
                  {isOpen ? (
                    <IoMdArrowDropup className="w-5 h-5" />
                  ) : (
                    <IoMdArrowDropdown className="w-5 h-5" />
                  )}
                </Button>
              </div>
            </TableCell>
            <TableCell>
              <Balance amount={loan.balance.collateral.btcBalance} currency="btc" />
            </TableCell>
            <TableCell>
              <Balance amount={loan.balance.interestIncurred.usdBalance} currency="usd" />
            </TableCell>
            <TableCell>
              <Balance amount={loan.balance.outstanding.usdBalance} currency="usd" />
            </TableCell>
            <TableCell>
              <LoanBadge status={loan.status} />
            </TableCell>
            <TableCell>
              <DropdownMenu>
                <DropdownMenuTrigger>
                  <Button variant="ghost">
                    <IoEllipsisHorizontal className="w-4 h-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent className="text-sm">
                  <DropdownMenuItem onClick={(e) => e.preventDefault()}>
                    <Link href={`/loan?loanId=${loan.loanId}`}>
                      <span>View Details</span>
                    </Link>
                  </DropdownMenuItem>
                  {loan.status === LoanStatus.Active && (
                    <DropdownMenuItem onClick={(e) => e.preventDefault()}>
                      <LoanPartialPaymentDialog refetch={refetch} loanId={loan.loanId}>
                        <span>Repayment</span>
                      </LoanPartialPaymentDialog>
                    </DropdownMenuItem>
                  )}
                  {loan.status === LoanStatus.New && (
                    <DropdownMenuItem onClick={(e) => e.preventDefault()}>
                      <LoanApproveDialog refetch={refetch} loanId={loan.loanId}>
                        <span>Approve Loan</span>
                      </LoanApproveDialog>
                    </DropdownMenuItem>
                  )}
                </DropdownMenuContent>
              </DropdownMenu>
            </TableCell>
          </TableRow>
        </CollapsibleTrigger>
        <TableCell colSpan={6} className="p-0">
          <CollapsibleContent asChild>
            <LoanTermsCollapsible loanTerms={loan.loanTerms} />
          </CollapsibleContent>
        </TableCell>
      </>
    </Collapsible>
  )
}

const LoanTermsCollapsible = ({ loanTerms }: { loanTerms: Loan["loanTerms"] }) => {
  return (
    <div className="p-4 w-full bg-secondary-foreground">
      <div className="grid grid-cols-2 gap-4">
        <div className="grid ">
          <DetailItem
            label="Duration"
            value={
              String(loanTerms.duration.units) +
              " " +
              formatPeriod(loanTerms.duration.period)
            }
          />
          <DetailItem label="Interval" value={formatInterval(loanTerms.interval)} />
          <DetailItem label="Annual Rate" value={`${loanTerms.annualRate}%`} />
        </div>
        <div className="grid ">
          <DetailItem label="Liquidation CVL" value={`${loanTerms.liquidationCvl}%`} />
          <DetailItem label="Margin Call CVL" value={`${loanTerms.marginCallCvl}%`} />
          <DetailItem label="Initial CVL" value={`${loanTerms.initialCvl}%`} />
        </div>
      </div>
    </div>
  )
}
