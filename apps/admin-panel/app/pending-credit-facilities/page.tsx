"use client"

import { useTranslations } from "next-intl"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import PendingCreditFacilities from "./list"

const PendingCreditFacilitiesPage: React.FC = () => {
  const t = useTranslations("PendingCreditFacilities")

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <PendingCreditFacilities />
      </CardContent>
    </Card>
  )
}

export default PendingCreditFacilitiesPage
