"use client"

import React from "react"
import { useTranslations } from "next-intl"

import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetCustomerBasicDetailsQuery } from "@/lib/graphql/generated"

type CustomerAccountBalancesProps = {
  balance: NonNullable<
    NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>["depositAccount"]
  >["balance"]
  publicId: string
}

export const CustomerAccountBalances: React.FC<CustomerAccountBalancesProps> = ({
  balance,
  publicId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.accountBalances")

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
      className="w-full md:w-1/2"
      publicId={publicId}
    />
  )
}
