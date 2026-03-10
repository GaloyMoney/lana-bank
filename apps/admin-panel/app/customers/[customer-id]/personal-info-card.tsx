"use client"

import { useTranslations } from "next-intl"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetCustomerBasicDetailsQuery } from "@/lib/graphql/generated"

type CustomerPersonalInfoCardProps = {
  customer: NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>
}

export const CustomerPersonalInfoCard: React.FC<CustomerPersonalInfoCardProps> = ({
  customer,
}) => {
  const t = useTranslations("Customers.CustomerDetails")
  const personalInfo = customer.personalInfo

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
    <div>
      <div className="px-4 py-2 border-b ">
        <h2 className="text-lg font-semibold">{t("personalInfo.title")}</h2>
      </div>
      <div className="p-4">
        <DetailsCard
          details={details}
          className="w-full"
          columns={2}
          variant="container"
        />
      </div>
    </div>
  )
}
