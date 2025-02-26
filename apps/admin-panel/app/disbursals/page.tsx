"use client"

import { useTranslations } from "next-intl"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import DisbursalsList from "./list"

const Disbursals: React.FC = () => {
  const t = useTranslations("Disbursals")

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <DisbursalsList />
      </CardContent>
    </Card>
  )
}

export default Disbursals
