"use client"

import { useTranslations } from "next-intl"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { GetCustomerBasicDetailsQuery } from "@/lib/graphql/generated"

type CustomerCompanyInfoCardProps = {
  customer: NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>
}

export const CustomerCompanyInfoCard: React.FC<CustomerCompanyInfoCardProps> = ({
  customer,
}) => {
  const t = useTranslations("Customers.CustomerDetails")
  const personalInfo = customer.personalInfo

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
