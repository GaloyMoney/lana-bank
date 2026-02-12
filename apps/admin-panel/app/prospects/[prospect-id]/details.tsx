"use client"

import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { CustomerType, GetProspectBasicDetailsQuery } from "@/lib/graphql/generated"

type ProspectDetailsCardProps = {
  prospect: NonNullable<GetProspectBasicDetailsQuery["prospectByPublicId"]>
}

export const ProspectDetailsCard: React.FC<ProspectDetailsCardProps> = ({
  prospect,
}) => {
  const t = useTranslations("Prospects.ProspectDetails.details")

  const getCustomerTypeDisplay = (customerType: CustomerType) => {
    switch (customerType) {
      case CustomerType.Individual:
        return t("customerType.individual")
      case CustomerType.GovernmentEntity:
        return t("customerType.governmentEntity")
      case CustomerType.PrivateCompany:
        return t("customerType.privateCompany")
      case CustomerType.Bank:
        return t("customerType.bank")
      case CustomerType.FinancialInstitution:
        return t("customerType.financialInstitution")
      case CustomerType.ForeignAgencyOrSubsidiary:
        return t("customerType.foreignAgency")
      case CustomerType.NonDomiciledCompany:
        return t("customerType.nonDomiciledCompany")
      default:
        return customerType
    }
  }

  const details: DetailItemProps[] = [
    {
      label: t("labels.email"),
      value: prospect.email,
    },
    {
      label: t("labels.telegram"),
      value: prospect.telegramId,
    },
    { label: t("labels.createdOn"), value: formatDate(prospect.createdAt) },
    {
      label: t("labels.customerType"),
      value: getCustomerTypeDisplay(prospect.customerType),
    },
  ]

  return (
    <DetailsCard title={t("title")} details={details} className="w-full" columns={4} />
  )
}
