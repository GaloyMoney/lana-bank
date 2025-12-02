"use client"

import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import FiscalYearsList from "./list"

const FiscalYearsPage: React.FC = () => {
  const t = useTranslations("FiscalYears")

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <FiscalYearsList />
      </CardContent>
    </Card>
  )
}

export default FiscalYearsPage
