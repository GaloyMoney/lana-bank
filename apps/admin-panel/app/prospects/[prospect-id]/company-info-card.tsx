"use client"

import { useTranslations } from "next-intl"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetProspectBasicDetailsQuery } from "@/lib/graphql/generated"

type ProspectCompanyInfoCardProps = {
  prospect: NonNullable<GetProspectBasicDetailsQuery["prospectByPublicId"]>
}

export const ProspectCompanyInfoCard: React.FC<ProspectCompanyInfoCardProps> = ({
  prospect,
}) => {
  const t = useTranslations("Prospects.ProspectDetails")
  const personalInfo = prospect.personalInfo

  const details: DetailItemProps[] = [
    {
      label: t("details.labels.companyName"),
      value: personalInfo?.companyName ?? "-",
    },
    ...(personalInfo?.nationality
      ? [{ label: t("details.labels.nationality"), value: personalInfo.nationality }]
      : []),
    ...(personalInfo?.address
      ? [{ label: t("details.labels.address"), value: personalInfo.address }]
      : []),
  ]

  return (
    <DetailsCard
      title={t("companyInfo.title")}
      details={details}
      className="md:w-full"
      columns={3}
    />
  )
}
