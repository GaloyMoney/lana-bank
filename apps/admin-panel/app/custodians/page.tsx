"use client"

import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import CustodiansList from "./list"

const Custodians: React.FC = () => {
  const t = useTranslations("Custodians")

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <CustodiansList />
      </CardContent>
    </Card>
  )
}

export default Custodians
