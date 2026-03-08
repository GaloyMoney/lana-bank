"use client"

import { useTranslations } from "next-intl"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetProspectBasicDetailsQuery } from "@/lib/graphql/generated"

type ProspectPersonalInfoCardProps = {
  prospect: NonNullable<GetProspectBasicDetailsQuery["prospectByPublicId"]>
}

export const ProspectPersonalInfoCard: React.FC<ProspectPersonalInfoCardProps> = ({
  prospect,
}) => {
  const t = useTranslations("Prospects.ProspectDetails")
  const personalInfo = prospect.personalInfo

  const details: DetailItemProps[] = [
    {
      label: t("details.labels.firstName"),
      value: personalInfo?.firstName ?? "-",
    },
    {
      label: t("details.labels.lastName"),
      value: personalInfo?.lastName ?? "-",
    },
    ...(personalInfo?.dateOfBirth
      ? [{ label: t("details.labels.dateOfBirth"), value: personalInfo.dateOfBirth }]
      : []),
    ...(personalInfo?.nationality
      ? [{ label: t("details.labels.nationality"), value: personalInfo.nationality }]
      : []),
    ...(personalInfo?.address
      ? [{ label: t("details.labels.address"), value: personalInfo.address }]
      : []),
  ]

  return (
    <DetailsCard
      title={t("personalInfo.title")}
      details={details}
      className="md:w-[75%]"
      columns={3}
    />
  )
}
