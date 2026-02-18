import { useTranslations } from "next-intl"

import { CustomerType } from "@/lib/graphql/generated"

export const CustomerTypeBadge = ({ customerType }: { customerType: CustomerType }) => {
  const t = useTranslations("Customers.customerType")

  switch (customerType) {
    case CustomerType.Individual:
      return t("individual")
    case CustomerType.GovernmentEntity:
      return t("governmentEntity")
    case CustomerType.PrivateCompany:
      return t("privateCompany")
    case CustomerType.Bank:
      return t("bank")
    case CustomerType.FinancialInstitution:
      return t("financialInstitution")
    case CustomerType.ForeignAgencyOrSubsidiary:
      return t("foreignAgency")
    case CustomerType.NonDomiciledCompany:
      return t("nonDomiciledCompany")
    default: {
      const _: never = customerType
      return _
    }
  }
}
