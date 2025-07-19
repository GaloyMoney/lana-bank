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
  const t = useTranslations("LedgerAccounts")
  const router = useRouter()
  const [accountId, setAccountId] = useState("")

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    router.push(`/ledger-account/${accountId}`)
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit}>
          <Label htmlFor="ledger-accounts">{t("form.labels.accountSearch")}</Label>
          <div className="flex flex-wrap md:flex-nowrap">
            <Input
              id="ledger-accounts"
              type="string"
              className="mr-2"
              value={accountId}
              onChange={(e) => {
                const { value } = e.target
                setAccountId(value)
              }}
              required
              placeholder={t("form.placeholders.accountSearch")}
              data-testid="search-ledger-accounts"
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
