"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { Badge } from "@lana/web/ui/badge"

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

  const details: DetailItemProps[] = [
    {
      label: t("labels.checkingSettled"),
      value: <Balance amount={balance.settled} currency="usd" />,
    },
    {
      label: t("labels.pendingWithdrawals"),
      value: <Balance amount={balance.pending} currency="usd" />,
    },
  ]

  return (
    <DetailsCard
      title={t("title")}
      details={details}
      badge={<DepositAccountStatusBadge status={status} />}
      className="w-full md:w-1/2"
      publicId={publicId}
    />
  )
}

export const DepositAccountStatusBadge: React.FC<{ status: DepositAccountStatus }> = ({
  status,
}) => {
  const t = useTranslations("Customers.CustomerDetails.depositAccount.status")

  const getVariant = (status: DepositAccountStatus) => {
    switch (status) {
      case DepositAccountStatus.Active:
        return "success"
      case DepositAccountStatus.Frozen:
        return "destructive"
      case DepositAccountStatus.Inactive:
        return "secondary"
      default: {
        const exhaustiveCheck: never = status
        return exhaustiveCheck
      }
    }
  }

  return (
    <Badge className="px-1.5 py-0.5 text-xs" variant={getVariant(status)}>
      {t(status.toLowerCase()).toUpperCase()}
    </Badge>
  )
}
