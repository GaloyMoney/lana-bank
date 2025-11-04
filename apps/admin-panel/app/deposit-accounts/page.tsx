"use client"

import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import DepositAccountsList from "./list"

const DepositAccounts: React.FC = () => {
  const t = useTranslations("DepositAccounts")

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <DepositAccountsList />
        </CardContent>
      </Card>
    </>
  )
}

export default DepositAccounts
