"use client"

import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import PriceProvidersList from "./list"

const PriceProviders: React.FC = () => {
  const t = useTranslations("PriceProviders")

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <PriceProvidersList />
      </CardContent>
    </Card>
  )
}

export default PriceProviders
