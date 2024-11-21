"use client"

import { useState } from "react"
import { PiPencilSimpleLineLight } from "react-icons/pi"

import UpdateTelegramIdDialog from "./update-telegram-id"

import DetailsCard, { DetailItemType } from "@/components/details-card"
import { AccountStatus, GetCustomerQuery } from "@/lib/graphql/generated"
import { Badge } from "@/components/primitive/badge"
import { formatDate } from "@/lib/utils"

type CustomerDetailsCardProps = {
  customer: NonNullable<GetCustomerQuery["customer"]>
  refetch: () => void
}

export const CustomerDetailsCard: React.FC<CustomerDetailsCardProps> = ({
  customer,
  refetch,
}) => {
  const [openUpdateTelegramIdDialog, setOpenUpdateTelegramIdDialog] = useState(false)

  const details: DetailItemType[] = [
    { label: "Email", value: customer.email },
    { label: "Created on", value: formatDate(customer.createdAt) },
    {
      label: "Telegram",
      value: (
        <div className="flex items-center gap-2">
          {customer.telegramId}
          <PiPencilSimpleLineLight
            onClick={() => setOpenUpdateTelegramIdDialog(true)}
            className="w-5 h-5 cursor-pointer text-primary"
          />
        </div>
      ),
    },
    {
      label: "Status",
      value: (
        <Badge
          variant={customer.status === AccountStatus.Active ? "success" : "secondary"}
        >
          {customer.status}
        </Badge>
      ),
    },
  ]

  return (
    <>
      <DetailsCard title="Customer" details={details} className="w-full" />
      <UpdateTelegramIdDialog
        customerId={customer.customerId}
        openUpdateTelegramIdDialog={openUpdateTelegramIdDialog}
        setOpenUpdateTelegramIdDialog={() => setOpenUpdateTelegramIdDialog(false)}
        refetch={refetch}
      />
    </>
  )
}
