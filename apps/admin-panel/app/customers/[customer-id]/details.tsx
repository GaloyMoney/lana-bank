"use client"

import { useState } from "react"
import { PiPencilSimpleLineLight } from "react-icons/pi"
import { useTranslations } from "next-intl"

import { Badge } from "@lana/web/ui/badge"
import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import UpdateTelegramIdDialog from "./update-telegram-id"
import UpdateEmailDialog from "./update-email"
import FreezeDepositAccountDialog from "./freeze-deposit-account"

import { DetailsCard, DetailItemProps } from "@/components/details"
import {
  CustomerStatus,
  CustomerType,
  GetCustomerBasicDetailsQuery,
  DepositAccountStatus,
} from "@/lib/graphql/generated"

type CustomerDetailsCardProps = {
  customer: NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>
}

export const CustomerDetailsCard: React.FC<CustomerDetailsCardProps> = ({ customer }) => {
  const t = useTranslations("Customers.CustomerDetails.details")
  const freezeT = useTranslations("Customers.CustomerDetails.freezeDepositAccount")

  const [openUpdateTelegramIdDialog, setOpenUpdateTelegramIdDialog] = useState(false)
  const [openUpdateEmailDialog, setOpenUpdateEmailDialog] = useState(false)
  const [openFreezeDialog, setOpenFreezeDialog] = useState(false)

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
    {
      label: t("labels.email"),
      value: (
        <button
          type="button"
          className="flex items-center gap-2"
          onClick={() => setOpenUpdateEmailDialog(true)}
        >
          {customer.email}
          <PiPencilSimpleLineLight className="w-5 h-5 cursor-pointer text-primary" />
        </button>
      ),
    },
    { label: t("labels.createdOn"), value: formatDate(customer.createdAt) },
    {
      label: t("labels.status"),
      value: (
        <Badge
          variant={customer.status === CustomerStatus.Active ? "success" : "secondary"}
        >
          {customer.status === CustomerStatus.Active
            ? t("status.active")
            : t("status.inactive")}
        </Badge>
      ),
    },
    {
      label: t("labels.customerType"),
      value: getCustomerTypeDisplay(customer.customerType),
    },
    {
      label: t("labels.telegram"),
      value: (
        <button
          type="button"
          className="flex items-center gap-2"
          onClick={() => setOpenUpdateTelegramIdDialog(true)}
        >
          {customer.telegramId}
          <PiPencilSimpleLineLight className="w-5 h-5 cursor-pointer text-primary" />
        </button>
      ),
    },
  ]

  const footerContent =
    customer.depositAccount &&
    customer.depositAccount.status === DepositAccountStatus.Active ? (
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          onClick={() => setOpenFreezeDialog(true)}
          disabled={!customer.depositAccount}
        >
          {freezeT("buttons.freezeDepositAccount")}
        </Button>
      </div>
    ) : undefined

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        className="w-full"
        columns={3}
        footerContent={footerContent}
      />
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
      {customer.depositAccount && (
        <FreezeDepositAccountDialog
          depositAccountId={customer.depositAccount.depositAccountId}
          balance={customer.depositAccount.balance}
          openFreezeDialog={openFreezeDialog}
          setOpenFreezeDialog={setOpenFreezeDialog}
        />
      )}
    </>
  )
}
