"use client"

import { useState } from "react"
import { PiPencilSimpleLineLight } from "react-icons/pi"

import UpdateTelegramIdDialog from "./update-telegram-id"

import { DetailItem, DetailsGroup } from "@/components/details"
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"

import { CreateLoanDialog } from "@/app/loans/create"
import { Button } from "@/components/primitive/button"
import { RecordDepositDialog } from "@/app/deposits/record"
import { WithdrawalInitiateDialog } from "@/app/withdrawals/initiate"

import { GetCustomerQuery, useMeQuery } from "@/lib/graphql/generated"

type CustomerDetailsCardProps = {
  customer: NonNullable<GetCustomerQuery["customer"]>
  refetch: () => void
}

export const CustomerDetailsCard: React.FC<CustomerDetailsCardProps> = ({
  customer,
  refetch,
}) => {
  const [openWithdrawalInitiateDialog, setOpenWithdrawalInitiateDialog] = useState(false)
  const [openRecordDepositDialog, setOpenRecordDepositDialog] = useState(false)
  const [openUpdateTelegramIdDialog, setOpenUpdateTelegramIdDialog] = useState(false)

  const { data } = useMeQuery()
  const canCreateLoan = data?.me.canCreateLoan || false

  return (
    <>
      <Card>
        <CardHeader className="pb-4">
          <div className="flex justify-between items-center">
            <CardTitle>Customer Overview</CardTitle>
          </div>
        </CardHeader>
        <div className="flex w-full items-center justify-between">
          <CardContent className="flex-1">
            <DetailsGroup>
              <DetailItem label="Customer ID" value={customer.customerId} />
              <DetailItem label="Email" value={customer.email} />
              <DetailItem
                label="Telegram"
                value={customer.telegramId}
                valueComponent={
                  <div className="flex items-center gap-2">
                    {customer.telegramId}
                    <PiPencilSimpleLineLight
                      onClick={() => setOpenUpdateTelegramIdDialog(true)}
                      className="w-5 h-5 cursor-pointer text-primary"
                    />
                  </div>
                }
              />
            </DetailsGroup>
          </CardContent>
          <CardFooter className="flex space-x-4 justify-end">
            {canCreateLoan && (
              <CreateLoanDialog refetch={refetch} customerId={customer.customerId}>
                <Button>New Loan</Button>
              </CreateLoanDialog>
            )}
            <Button onClick={() => setOpenRecordDepositDialog(true)}>
              Record Deposit
            </Button>
            <Button onClick={() => setOpenWithdrawalInitiateDialog(true)}>
              Record Withdrawal
            </Button>
          </CardFooter>
        </div>
      </Card>
      {openWithdrawalInitiateDialog && (
        <WithdrawalInitiateDialog
          customerId={customer.customerId}
          openWithdrawalInitiateDialog={openWithdrawalInitiateDialog}
          setOpenWithdrawalInitiateDialog={() => setOpenWithdrawalInitiateDialog(false)}
        />
      )}
      {openRecordDepositDialog && (
        <RecordDepositDialog
          customerId={customer.customerId}
          openRecordDepositDialog={openRecordDepositDialog}
          setOpenRecordDepositDialog={() => setOpenRecordDepositDialog(false)}
        />
      )}
      {
        <UpdateTelegramIdDialog
          customerId={customer.customerId}
          openUpdateTelegramIdDialog={openUpdateTelegramIdDialog}
          setOpenUpdateTelegramIdDialog={() => setOpenUpdateTelegramIdDialog(false)}
          refetch={refetch}
        />
      }
    </>
  )
}
