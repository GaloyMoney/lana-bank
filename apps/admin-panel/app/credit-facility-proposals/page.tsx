"use client"

import { useTranslations } from "next-intl"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import CreditFacilityProposalsList from "./list"

const CreditFacilityProposals: React.FC = () => {
  const t = useTranslations("CreditFacilityProposals")

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <CreditFacilityProposalsList />
      </CardContent>
    </Card>
  )
}

export default CreditFacilityProposals
