"use client"
import { DetailItemProps, DetailsCard } from "@lana/web/components/details"
import { Badge } from "@lana/web/ui/badge"

import React from "react"

import { useBreakpointDown } from "@lana/web/hooks"

import { formatDate } from "@lana/web/utils"

import { KycVerification, MeQuery } from "@/lib/graphql/generated"
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
      label: "KYC",
      value: (
        <Badge
          variant={
            customer.kycVerification === KycVerification.Verified
              ? "success"
              : "secondary"
          }
        >
          {customer.kycVerification === KycVerification.Verified
            ? "Verified"
            : customer.kycVerification === KycVerification.NoKyc
              ? "No KYC"
              : "Rejected"}
        </Badge>
      ),
    },
    {
      label: "Telegram",
      value: customer.telegramHandle,
    },
    {
      label: "Joined on",
      value: formatDate(customer.createdAt),
    },
  ]

  const name = `${customer.personalInfo.firstName} ${customer.personalInfo.lastName}`

  return (
    <DetailsCard
      title={<div className="text-md font-semibold text-primary">{name}</div>}
      details={details}
    />
  )
}

export default UserDetailsCard
