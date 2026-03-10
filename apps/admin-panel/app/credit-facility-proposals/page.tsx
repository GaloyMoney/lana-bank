"use client"

import { useTranslations } from "next-intl"

import CreditFacilityProposalsList from "./list"

import CreateButton from "@/app/create"
import { PageHeader } from "@/components/page-header"

const CreditFacilityProposals: React.FC = () => {
  const t = useTranslations("CreditFacilityProposals")

  return (
    <div className="border-l border-r flex-1">
      <PageHeader
        title={t("title")}
        description={t("description")}
        actions={<CreateButton />}
        showBreadcrumb={false}
      />
      <CreditFacilityProposalsList />
    </div>
  )
}

export default CreditFacilityProposals
