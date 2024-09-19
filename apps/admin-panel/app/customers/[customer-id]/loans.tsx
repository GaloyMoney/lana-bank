"use client"

import { useState } from "react"
import Link from "next/link"

import { IoEllipsisHorizontal } from "react-icons/io5"
import { IoMdArrowDropdown, IoMdArrowDropup } from "react-icons/io"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { Button } from "@/components/primitive/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/primitive/dropdown-menu"
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/primitive/collapsible"
import { DetailItem } from "@/components/details"
import Balance from "@/components/balance/balance"

import { CollateralUpdateDialog } from "@/app/loans/update-collateral"
import { LoanStatusBadge } from "@/app/loans/status-badge"
import { LoanPartialPaymentDialog } from "@/app/loans/partial-payment"
import { LoanApproveDialog } from "@/app/loans/approve"

import { formatInterval, formatPeriod } from "@/lib/utils"
import { GetCustomerQuery, Loan, LoanStatus } from "@/lib/graphql/generated"

type CustomerLoansTableProps = {
  loans: NonNullable<GetCustomerQuery["customer"]>["loans"]
  refetch: () => void
}
export const CustomerLoansTable: React.FC<CustomerLoansTableProps> = ({
  loans,
  refetch,
}) => (
  <Card className="mt-4">
    <CardHeader className="flex flex-row justify-between items-center">
      <div className="flex flex-col space-y-1.5">
        <CardTitle>Loans</CardTitle>
      </div>
    </CardHeader>
    {loans.length === 0 ? (
      <CardContent>No loans found for this customer</CardContent>
    ) : (
      <CardContent>
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
            {loans.map((loan) => (
              <LoanRow key={loan.loanId} loan={loan} refetch={refetch} />
            ))}
          </TableBody>
        </Table>
      </CardContent>
    )}
  </Card>
)

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
const LoanRow = ({ loan, refetch }: { loan: LoanRowProps; refetch: () => void }) => {
  const [isOpen, setIsOpen] = useState(false)
  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] = useState<{
    loanId: string
    existingCollateral: number
  } | null>(null)

  return (
    <Collapsible asChild>
      <>
        {openCollateralUpdateDialog && (
          <CollateralUpdateDialog
            setOpenCollateralUpdateDialog={() => setOpenCollateralUpdateDialog(null)}
            openCollateralUpdateDialog={Boolean(openCollateralUpdateDialog)}
            loanData={{
              loanId: openCollateralUpdateDialog.loanId,
              existingCollateral: openCollateralUpdateDialog.existingCollateral,
            }}
            refetch={refetch}
          />
        )}
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
              <LoanStatusBadge status={loan.status} />
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
                    <Link href={`/loans/${loan.loanId}`}>
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
                      <LoanApproveDialog refetch={refetch} loanDetails={loan as Loan}>
                        <span>Approve Loan</span>
                      </LoanApproveDialog>
                    </DropdownMenuItem>
                  )}
                  {loan.status !== LoanStatus.Closed && (
                    <DropdownMenuItem
                      onClick={(e) => {
                        e.preventDefault()
                        setOpenCollateralUpdateDialog({
                          loanId: loan.loanId,
                          existingCollateral: loan.balance.collateral.btcBalance,
                        })
                      }}
                    >
                      Update Collateral
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
