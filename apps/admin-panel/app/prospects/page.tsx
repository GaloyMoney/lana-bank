"use client"

import { useTranslations } from "next-intl"

import CreateButton from "../create"

import ProspectsList from "./list"

import { PageHeader } from "@/components/page-header"

const Prospects: React.FC = () => {
  const t = useTranslations("Prospects")

  return (
    <div className="border-l border-r flex-1">
      <PageHeader
        title={t("title")}
        description={t("description")}
        actions={<CreateButton />}
        showBreadcrumb={false}
      />
      <ProspectsList />
    </div>
  )
}

export default Prospects
