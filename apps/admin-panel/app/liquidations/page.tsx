"use client"

import { useTranslations } from "next-intl"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import LiquidationsList from "./list"

const Liquidations: React.FC = () => {
  const t = useTranslations("Liquidations")

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <LiquidationsList />
      </CardContent>
    </Card>
  )
}

export default Liquidations


