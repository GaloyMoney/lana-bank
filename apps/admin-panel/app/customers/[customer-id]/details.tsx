"use client"

import { useState } from "react"
import { PiPencilSimpleLineLight } from "react-icons/pi"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import { Label } from "@lana/web/ui/label"

import { ActivityStatusBadge } from "../activity-status-badge"

import { CustomerTypeBadge } from "../customer-type-badge"

import UpdateTelegramHandleDialog from "./update-telegram-handle"
import UpdateEmailDialog from "./update-email"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetCustomerBasicDetailsQuery } from "@/lib/graphql/generated"

type CustomerDetailsCardProps = {
  customer: NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>
}

export const CustomerDetailsCard: React.FC<CustomerDetailsCardProps> = ({ customer }) => {
  const t = useTranslations("Customers.CustomerDetails.details")

  const [openUpdateTelegramHandleDialog, setOpenUpdateTelegramHandleDialog] = useState(false)
  const [openUpdateEmailDialog, setOpenUpdateEmailDialog] = useState(false)

  const personalInfo = customer.personalInfo

  const details: DetailItemProps[] = [
    {
      label: t("labels.firstName"),
      value: personalInfo?.firstName ?? "-",
    },
    {
      label: t("labels.lastName"),
      value: personalInfo?.lastName ?? "-",
    },
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
      label: t("labels.status"),
      value: <ActivityStatusBadge status={customer.activity} />,
    },
    {
      label: t("labels.customerType"),
      value: <CustomerTypeBadge customerType={customer.customerType} />,
    },
    ...(personalInfo?.dateOfBirth
      ? [{ label: t("labels.dateOfBirth"), value: personalInfo.dateOfBirth }]
      : []),
    ...(personalInfo?.nationality
      ? [{ label: t("labels.nationality"), value: personalInfo.nationality }]
      : []),
    ...(personalInfo?.address
      ? [{ label: t("labels.address"), value: personalInfo.address }]
      : []),
  ]

  return (
    <>
      <DetailsCard title={t("title")} details={details} className="w-full" columns={4} />
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
    </>
  )
}
