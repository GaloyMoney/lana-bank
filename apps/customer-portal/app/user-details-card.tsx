"use client"
import { DetailItemProps, DetailsCard } from "@lana/web/components/details"

import React from "react"

import { useBreakpointDown } from "@lana/web/hooks"

import { formatDate } from "@lana/web/utils"

import { MeQuery } from "@/lib/graphql/generated"
import Balance from "@/components/balance"

function UserDetailsCard({
  customer,
  totalBalanceInCents,
}: {
  customer: NonNullable<MeQuery["me"]["customer"]>
  totalBalanceInCents: number
}) {
  const isMobile = useBreakpointDown("md")

  const details: DetailItemProps[] = [
    ...(!isMobile
      ? [
          {
            label: "Balance",
            value: <Balance amount={totalBalanceInCents} currency="usd" />,
          },
        ]
      : []),
    {
      label: "Telegram",
      value: customer.telegramHandle,
    },
    {
      label: "Joined on",
      value: formatDate(customer.createdAt),
    },
  ]

  const name =
    customer.personalInfo?.companyName ??
    `${customer.personalInfo?.firstName ?? "-"} ${customer.personalInfo?.lastName ?? "-"}`

  return (
    <DetailsCard
      title={<div className="text-md font-semibold text-primary">{name}</div>}
      details={details}
    />
  )
}

export default UserDetailsCard
