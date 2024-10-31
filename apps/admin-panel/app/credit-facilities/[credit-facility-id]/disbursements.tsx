import React, { useState } from "react"
import { IoCheckmark } from "react-icons/io5"

import { DisbursementDetailsDialog } from "../disbursement-details"
import { CreditFacilityDisbursementConfirmDialog } from "../disbursement-approve"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import {
  DisbursementStatus,
  GetCreditFacilityDetailsQuery,
} from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import Balance from "@/components/balance/balance"
import { formatDate } from "@/lib/utils"

type Disbursement = NonNullable<
  GetCreditFacilityDetailsQuery["creditFacility"]
>["disbursements"][number]

type CreditFacilityDisbursementsProps = {
  creditFacility: NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>
}

export const CreditFacilityDisbursements: React.FC<CreditFacilityDisbursementsProps> = ({
  creditFacility,
}) => {
  const [selectedApprovalDisbursement, setSelectedApprovalDisbursement] =
    useState<Disbursement | null>(null)
  const [selectedDetailsDisbursement, setSelectedDetailsDisbursement] =
    useState<Disbursement | null>(null)

  const handleOpenApproveDialog = (disbursement: Disbursement) => {
    setSelectedApprovalDisbursement(disbursement)
  }

  const handleCloseApproveDialog = () => {
    setSelectedApprovalDisbursement(null)
  }

  const handleOpenDetailsDialog = (disbursement: Disbursement) => {
    setSelectedDetailsDisbursement(disbursement)
  }

  const handleCloseDetailsDialog = () => {
    setSelectedDetailsDisbursement(null)
  }

  return (
    <>
      <Card className="mt-4">
        <CardHeader>
          <CardTitle>Disbursements</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[30%]">ID</TableHead>
                <TableHead className="w-[20%]">Amount</TableHead>
                <TableHead className="w-[20%]">Created At</TableHead>
                <TableHead className="w-[20%] text-right">Action</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {creditFacility.disbursements.map((disbursement) => (
                <TableRow key={disbursement.id}>
                  <TableCell>{disbursement.id.split("disbursement:")[1]}</TableCell>
                  <TableCell>
                    <Balance amount={disbursement.amount} currency="usd" />
                  </TableCell>
                  <TableCell>{formatDate(disbursement.createdAt)}</TableCell>
                  <TableCell className="text-right">
                    {disbursement.status === DisbursementStatus.New ? (
                      <Button
                        className="px-2 py-1 text-primary"
                        variant="ghost"
                        onClick={() => handleOpenApproveDialog(disbursement)}
                      >
                        Approval Required
                      </Button>
                    ) : (
                      <Button
                        className="px-2 py-1 text-success"
                        variant="ghost"
                        onClick={() => handleOpenDetailsDialog(disbursement)}
                      >
                        <IoCheckmark className="h-4 w-4 mr-1" /> Approved
                      </Button>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {selectedApprovalDisbursement && (
        <CreditFacilityDisbursementConfirmDialog
          setOpenDialog={handleCloseApproveDialog}
          openDialog={true}
          creditFacilityId={creditFacility.creditFacilityId}
          disbursementIdx={selectedApprovalDisbursement.index}
          disbursement={selectedApprovalDisbursement}
        />
      )}

      {selectedDetailsDisbursement && (
        <DisbursementDetailsDialog
          setOpenDialog={handleCloseDetailsDialog}
          openDialog={true}
          disbursement={selectedDetailsDisbursement}
        />
      )}
    </>
  )
}
