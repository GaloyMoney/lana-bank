import React from "react"

import { useTranslations } from "next-intl"

import { KycStatusBadge } from "@/app/customers/kyc-status-badge"

import { KycLevel } from "@/lib/graphql/generated"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { removeUnderscore } from "@/lib/utils"

type KycStatusProps = {
  level: KycLevel
  applicantId: string | null | undefined
}

export const KycStatus: React.FC<KycStatusProps> = ({
  level,
  applicantId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.kycStatus")

  const details: DetailItemProps[] = [
    {
      label: t("labels.level"),
      value: removeUnderscore(level),
    },
    ...(applicantId
      ? [
          {
            label: t("labels.kycApplicationLink"),
            value: (
              <a
                href={`https://cockpit.sumsub.com/checkus#/applicant/${applicantId}/client/basicInfo`}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-500 underline"
              >
                {applicantId}
              </a>
            ),
          },
        ]
      : []),
  ]

  return (
    <DetailsCard
      title={t("title")}
      badge={<KycStatusBadge level={level} />}
      details={details}
      className="w-full md:w-[25%]"
      columns={1}
    />
  )
}
