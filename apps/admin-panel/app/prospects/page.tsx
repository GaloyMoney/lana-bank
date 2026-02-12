"use client"

import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import ProspectsList from "./list"

const Prospects: React.FC = () => {
  const t = useTranslations("Prospects")

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <ProspectsList />
        </CardContent>
      </Card>
    </>
  )
}

export default Prospects
