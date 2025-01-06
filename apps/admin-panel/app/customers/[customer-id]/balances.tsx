"use client"

import React from "react"

import Balance from "@/components/balance/balance"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetCustomerOverviewQuery } from "@/lib/graphql/generated"

type CustomerAccountBalancesProps = {
  balance: NonNullable<GetCustomerOverviewQuery["customer"]>["depositAccount"]["balance"]
}

export const CustomerAccountBalances: React.FC<CustomerAccountBalancesProps> = ({
  balance,
}) => {
  const details: DetailItemProps[] = [
    {
      label: "Checking Settled Balance (USD)",
      value: <Balance amount={balance.settled} currency="usd" />,
    },
    {
      label: "Pending Withdrawals (USD)",
      value: <Balance amount={balance.pending} currency="usd" />,
    },
  ]

  return (
    <DetailsCard
      title="Account Balances"
      description="Balance Details for this Customer"
      details={details}
      className="w-full md:w-1/2"
    />
  )
}
