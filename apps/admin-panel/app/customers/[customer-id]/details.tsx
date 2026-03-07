"use client"

import { useState } from "react"
import { PiPencilSimpleLineLight } from "react-icons/pi"
import { XCircle } from "lucide-react"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import { Label } from "@lana/web/ui/label"
import { Button } from "@lana/web/ui/button"

import { CustomerStatusBadge } from "../customer-status-badge"

import { CustomerTypeBadge } from "../customer-type-badge"

import UpdateTelegramHandleDialog from "./update-telegram-handle"
import UpdateEmailDialog from "./update-email"
import CloseCustomerDialog from "./close-customer"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetCustomerBasicDetailsQuery, CustomerStatus } from "@/lib/graphql/generated"

type CustomerDetailsCardProps = {
  customer: NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>
}

export const CustomerDetailsCard: React.FC<CustomerDetailsCardProps> = ({ customer }) => {
  const t = useTranslations("Customers.CustomerDetails.details")

  const [openUpdateTelegramHandleDialog, setOpenUpdateTelegramHandleDialog] = useState(false)
  const [openUpdateEmailDialog, setOpenUpdateEmailDialog] = useState(false)
  const [openCloseDialog, setOpenCloseDialog] = useState(false)

  const details: DetailItemProps[] = [
    {
      label: (
        <Label className="flex items-center font-semibold">
          <span>{t("labels.email")}</span>
          <div className="cursor-pointer text-primary px-1">
            <PiPencilSimpleLineLight
              onClick={() => setOpenUpdateEmailDialog(true)}
              className="w-4 h-4"
            />
          </div>
        </Label>
      ),
      value: customer.email,
    },
    {
      label: (
        <Label className="flex items-center font-semibold">
          <span>{t("labels.telegram")}</span>
          <div className="cursor-pointer text-primary px-1">
            <PiPencilSimpleLineLight
              onClick={() => setOpenUpdateTelegramHandleDialog(true)}
              className="w-4 h-4"
            />
          </div>
        </Label>
      ),
      value: customer.telegramHandle,
    },
    { label: t("labels.createdOn"), value: formatDate(customer.createdAt) },
    {
      label: t("labels.customerStatus"),
      value: <CustomerStatusBadge status={customer.status} />,
    },
    {
      label: t("labels.customerType"),
      value: <CustomerTypeBadge customerType={customer.customerType} />,
    },
  ]

  const footerContent = customer.status !== CustomerStatus.Closed && (
    <Button variant="destructive" onClick={() => setOpenCloseDialog(true)}>
      <XCircle />
      {t("buttons.close")}
    </Button>
  )

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        className="w-full"
        columns={4}
        footerContent={footerContent || undefined}
      />
      <UpdateTelegramHandleDialog
        customerId={customer.customerId}
        openUpdateTelegramHandleDialog={openUpdateTelegramHandleDialog}
        setOpenUpdateTelegramHandleDialog={setOpenUpdateTelegramHandleDialog}
      />
      <UpdateEmailDialog
        customerId={customer.customerId}
        openUpdateEmailDialog={openUpdateEmailDialog}
        setOpenUpdateEmailDialog={setOpenUpdateEmailDialog}
      />
      <CloseCustomerDialog
        customerId={customer.customerId}
        openCloseDialog={openCloseDialog}
        setOpenCloseDialog={setOpenCloseDialog}
      />
    </>
  )
}
