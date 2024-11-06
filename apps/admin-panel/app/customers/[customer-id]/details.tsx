"use client"

import { useState } from "react"
import { PiPencilSimpleLineLight } from "react-icons/pi"

import UpdateTelegramIdDialog from "./update-telegram-id"

import { DetailItem, DetailsGroup } from "@/components/details"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"

import { Button } from "@/components/primitive/button"
import { RecordDepositDialog } from "@/app/deposits/record"
import { WithdrawalInitiateDialog } from "@/app/withdrawals/initiate"

import { GetCustomerQuery } from "@/lib/graphql/generated"
import { CreateCreditFacilityDialog } from "@/app/credit-facilities/create"

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
  const [openCreateCreditFacilityDialog, setOpenCreateCreditFacilityDialog] =
    useState(false)

  return (
    <div className="flex gap-4">
      <Card className="w-11/12">
        <CardHeader className="pb-4">
          <div className="flex justify-between items-center">
            <CardTitle>Customer</CardTitle>
          </div>
        </CardHeader>
        <div className="flex w-full items-center justify-between">
          <CardContent className="flex-1">
            <DetailsGroup>
              <DetailItem label="Customer ID" value={customer.customerId} />
              <DetailItem label="Email" value={customer.email} />
              <DetailItem
                label="Telegram"
                value={
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
        </div>
      </Card>
      <div className="flex flex-col space-y-2 mt-1">
        {customer.subjectCanRecordDeposit && (
          <Button onClick={() => setOpenRecordDepositDialog(true)}>Record Deposit</Button>
        )}
        {customer.subjectCanInitiateWithdrawal && (
          <Button onClick={() => setOpenWithdrawalInitiateDialog(true)}>
            Initiate Withdrawal
          </Button>
        )}
        {customer.subjectCanCreateCreditFacility && (
          <Button
            className="w-full"
            onClick={() => setOpenCreateCreditFacilityDialog(true)}
          >
            New Credit Facility
          </Button>
        )}
      </div>
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
      <UpdateTelegramIdDialog
        customerId={customer.customerId}
        openUpdateTelegramIdDialog={openUpdateTelegramIdDialog}
        setOpenUpdateTelegramIdDialog={() => setOpenUpdateTelegramIdDialog(false)}
        refetch={refetch}
      />
      <CreateCreditFacilityDialog
        customerId={customer.customerId}
        openCreateCreditFacilityDialog={openCreateCreditFacilityDialog}
        setOpenCreateCreditFacilityDialog={setOpenCreateCreditFacilityDialog}
      />
    </div>
  )
}
