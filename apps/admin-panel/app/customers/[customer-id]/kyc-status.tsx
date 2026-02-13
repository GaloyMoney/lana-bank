import React from "react"

import { useTranslations } from "next-intl"

import { KycStatusBadge } from "@/app/customers/kyc-status-badge"

import { KycLevel, KycVerification } from "@/lib/graphql/generated"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { removeUnderscore } from "@/lib/utils"

type KycStatusProps = {
  kycVerification: KycVerification
  level: KycLevel
  applicantId: string
}

export const KycStatus: React.FC<KycStatusProps> = ({
  kycVerification,
  level,
  applicantId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.kycStatus")

  const sumsubLink = `https://cockpit.sumsub.com/checkus#/applicant/${applicantId}/client/basicInfo`

  const details: DetailItemProps[] = [
    {
      label: t("labels.level"),
      value: removeUnderscore(level),
    },
    {
      label: t("labels.kycApplicationLink"),
      value: (
        <a
          href={sumsubLink}
          target="_blank"
          rel="noopener noreferrer"
          className="text-blue-500 underline"
        >
          {applicantId}
        </a>
      ),
    },
  ]

  return (
    <DetailsCard
      title={t("title")}
      badge={<KycStatusBadge status={kycVerification} />}
      details={details}
      className="w-full md:w-1/4"
      columns={1}
    />
  )
}
