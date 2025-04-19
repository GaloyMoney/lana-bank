"use client"

import React, { useState } from "react"

import { useRouter } from "next/navigation"

import { Input } from "@lana/web/ui/input"
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"
import { useTranslations } from "next-intl"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

export default function LedgerAccount() {
  const t = useTranslations("LedgerTransaction")
  const router = useRouter()
  const [transactionId, setTransactionId] = useState("")

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    router.push(`/ledger-account/${transactionId}`)
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit}>
          <Label htmlFor="ledger-transactions">{t("form.labels.transaction")}</Label>
          <div className="flex flex-wrap md:flex-nowrap">
            <Input
              id="ledger-transactions"
              type="string"
              className="mr-2"
              value={transactionId}
              onChange={(e) => {
                const { value } = e.target
                setTransactionId(value)
              }}
              required
              placeholder={t("form.placeholders.transaction")}
              data-testid="search-ledger-transaction"
            />
            <Button type="submit" className="mt-2 md:mt-0">
              Search
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}
