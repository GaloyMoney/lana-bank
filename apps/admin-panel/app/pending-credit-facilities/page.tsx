"use client"

import { useTranslations } from "next-intl"

import PendingCreditFacilities from "./list"

import CreateButton from "@/app/create"
import { PageHeader } from "@/components/page-header"

const PendingCreditFacilitiesPage: React.FC = () => {
  const t = useTranslations("PendingCreditFacilities")

  return (
    <div className="border-l border-r flex-1">
      <PageHeader
        title={t("title")}
        description={t("description")}
        actions={<CreateButton />}
        showBreadcrumb={false}
      />
      <PendingCreditFacilities />
    </div>
  )
}

export default PendingCreditFacilitiesPage
