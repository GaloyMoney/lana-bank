import React, { useState } from "react"
import { IoCheckmark } from "react-icons/io5"

import Link from "next/link"

import { DisbursalDetailsDialog } from "../disbursal-details"
import { CreditFacilityDisbursalApproveDialog } from "../disbursal-approve"

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
  ApprovalProcessStatus,
  DisbursalStatus,
  GetCreditFacilityDetailsQuery,
} from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import Balance from "@/components/balance/balance"
import { formatDate } from "@/lib/utils"

type Disbursal = NonNullable<
  GetCreditFacilityDetailsQuery["creditFacility"]
>["disbursals"][number]

type CreditFacilityDisbursalsProps = {
  creditFacility: NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>
  refetch: () => void
}

export const CreditFacilityDisbursals: React.FC<CreditFacilityDisbursalsProps> = ({
  creditFacility,
  refetch,
}) => {
  const [selectedDetailsDisbursal, setSelectedDetailsDisbursal] =
    useState<Disbursal | null>(null)
  const [selectedApprovalProcessDisbursal, setSelectedApprovalProcessDisbursal] =
    useState<Disbursal | null>(null)

  const handleOpenApprovalProcessDialog = (disbursal: Disbursal) => {
    setSelectedApprovalProcessDisbursal(disbursal)
  }

  const handleCloseApprovalProcessDialog = () => {
    setSelectedApprovalProcessDisbursal(null)
  }

  const handleOpenDetailsDialog = (disbursal: Disbursal) => {
    setSelectedDetailsDisbursal(disbursal)
  }

  const handleCloseDetailsDialog = () => {
    setSelectedDetailsDisbursal(null)
  }

  return (
    <>
      <Card className="mt-4">
        <CardHeader>
          <CardTitle>Disbursals</CardTitle>
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
              {creditFacility.disbursals.map((disbursal) => (
                <TableRow key={disbursal.id}>
                  <TableCell className="cursor-pointer hover:underline">
                    <Link href={`/disbursals/${disbursal.id.split(":")[1]}`}>
                      {disbursal.id.split(":")[1]}
                    </Link>
                  </TableCell>
                  <TableCell>
                    <Balance amount={disbursal.amount} currency="usd" />
                  </TableCell>
                  <TableCell>{formatDate(disbursal.createdAt)}</TableCell>
                  <TableCell className="text-right">
                    {disbursal.status === DisbursalStatus.New &&
                    disbursal.approvalProcess.status ===
                      ApprovalProcessStatus.InProgress ? (
                      <Button
                        className="px-2 py-1 text-primary"
                        variant="outline"
                        onClick={() => handleOpenApprovalProcessDialog(disbursal)}
                      >
                        Approval Required
                      </Button>
                    ) : (
                      <Button
                        className="px-2 py-1 text-success"
                        variant="outline"
                        onClick={() => handleOpenDetailsDialog(disbursal)}
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

      {selectedApprovalProcessDisbursal && (
        <CreditFacilityDisbursalApproveDialog
          setOpenDialog={handleCloseApprovalProcessDialog}
          openDialog={true}
          creditFacilityId={creditFacility.creditFacilityId}
          disbursalIdx={selectedApprovalProcessDisbursal.index}
          disbursal={selectedApprovalProcessDisbursal}
          refetch={refetch}
        />
      )}

      {selectedDetailsDisbursal && (
        <DisbursalDetailsDialog
          setOpenDialog={handleCloseDetailsDialog}
          openDialog={true}
          disbursal={selectedDetailsDisbursal}
        />
      )}
    </>
  )
}
