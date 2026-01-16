"use client"

import { useState } from "react"
import { PiPencilSimpleLineLight } from "react-icons/pi"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import { Label } from "@lana/web/ui/label"

import { ActivityStatusBadge } from "../activity-status-badge"

import UpdateTelegramIdDialog from "./update-telegram-id"
import UpdateEmailDialog from "./update-email"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { CustomerType, GetCustomerBasicDetailsQuery } from "@/lib/graphql/generated"

type CustomerDetailsCardProps = {
  customer: NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>
}

export const CustomerDetailsCard: React.FC<CustomerDetailsCardProps> = ({ customer }) => {
  const t = useTranslations("Customers.CustomerDetails.details")

  const [openUpdateTelegramIdDialog, setOpenUpdateTelegramIdDialog] = useState(false)
  const [openUpdateEmailDialog, setOpenUpdateEmailDialog] = useState(false)

  const getCustomerTypeDisplay = (customerType: CustomerType) => {
    switch (customerType) {
      case CustomerType.Individual:
        return t("customerType.individual")
      case CustomerType.GovernmentEntity:
        return t("customerType.governmentEntity")
      case CustomerType.PrivateCompany:
        return t("customerType.privateCompany")
      case CustomerType.Bank:
        return t("customerType.bank")
      case CustomerType.FinancialInstitution:
        return t("customerType.financialInstitution")
      case CustomerType.ForeignAgencyOrSubsidiary:
        return t("customerType.foreignAgency")
      case CustomerType.NonDomiciledCompany:
        return t("customerType.nonDomiciledCompany")
      default:
        return customerType
    }
  }

  const details: DetailItemProps[] = [
    ...(customer.fullName
      ? [{ label: t("labels.fullName"), value: customer.fullName }]
      : []),
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
              onClick={() => setOpenUpdateTelegramIdDialog(true)}
              className="w-4 h-4"
            />
          </div>
        </Label>
      ),
      value: customer.telegramId,
    },
    ...(customer.dateOfBirth
      ? [{ label: t("labels.dateOfBirth"), value: customer.dateOfBirth }]
      : []),
    ...(customer.country
      ? [{ label: t("labels.country"), value: customer.country }]
      : []),
    { label: t("labels.createdOn"), value: formatDate(customer.createdAt) },
    {
      label: t("labels.status"),
      value: <ActivityStatusBadge status={customer.activity} />,
    },
    {
      label: t("labels.customerType"),
      value: getCustomerTypeDisplay(customer.customerType),
    },
  ]

  return (
    <>
      <DetailsCard title={t("title")} details={details} className="w-full" columns={4} />
      <UpdateTelegramIdDialog
        customerId={customer.customerId}
        openUpdateTelegramIdDialog={openUpdateTelegramIdDialog}
        setOpenUpdateTelegramIdDialog={setOpenUpdateTelegramIdDialog}
      />
      <UpdateEmailDialog
        customerId={customer.customerId}
        openUpdateEmailDialog={openUpdateEmailDialog}
        setOpenUpdateEmailDialog={setOpenUpdateEmailDialog}
      />
    </>
  )
}
