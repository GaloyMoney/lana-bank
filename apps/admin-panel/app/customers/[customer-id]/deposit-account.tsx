"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { useRouter } from "next/navigation"
import { ArrowRight } from "lucide-react"

import { Button } from "@lana/web/ui/button"

import { DepositAccountStatusBadge } from "@/app/deposit-accounts/status-badge"

import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"
import {
  DepositAccountStatus,
  GetCustomerBasicDetailsQuery,
} from "@/lib/graphql/generated"

type DepositAccountProps = {
  balance: NonNullable<
    NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>["depositAccount"]
  >["balance"]
  publicId: string
  status: DepositAccountStatus
}

export const DepositAccount: React.FC<DepositAccountProps> = ({
  balance,
  publicId,
  status,
}) => {
  const t = useTranslations("Customers.CustomerDetails.depositAccount")
  const router = useRouter()

  const handleViewDetails = () => {
    router.push(`/deposit-accounts/${publicId}`)
  }

  const details: DetailItemProps[] = [
    {
      label: t("labels.checkingSettled"),
      value: <Balance amount={balance.settled} currency="usd" />,
    },
    {
      label: t("labels.pendingWithdrawals"),
      value: <Balance amount={balance.pending} currency="usd" />,
    },
    {
      label: t("labels.status"),
      value: <DepositAccountStatusBadge status={status} />,
    },
  ]

  return (
    <DetailsCard
      title={t("title")}
      details={details}
      columns={3}
      className="w-full md:w-3/4"
      publicId={publicId}
      footerContent={
        <Button variant="outline" onClick={handleViewDetails}>
          {t("buttons.viewDetails")}
          <ArrowRight />
        </Button>
      }
    />
  )
}
