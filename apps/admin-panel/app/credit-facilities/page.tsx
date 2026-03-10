"use client"

import { useTranslations } from "next-intl"

import CustomersList from "./list"

import CreateButton from "@/app/create"
import { PageHeader } from "@/components/page-header"

const CreditFacilities: React.FC = () => {
  const t = useTranslations("CreditFacilities")

  return (
    <div className="border-l border-r flex-1">
      <PageHeader
        title={t("title")}
        description={t("description")}
        actions={<CreateButton />}
        showBreadcrumb={false}
      />
      <CustomersList />
    </div>
  )
}

export default CreditFacilities
