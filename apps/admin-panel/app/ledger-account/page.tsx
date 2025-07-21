"use client"

import React, { useState } from "react"
import { useRouter } from "next/navigation"
import { useTranslations } from "next-intl"
import { Search } from "lucide-react"
import { gql } from "@apollo/client"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Input } from "@lana/web/ui/input"
import { Button } from "@lana/web/ui/button"

import { isUUID } from "@/lib/utils"
import {
  useLedgerAccountExistsByCodeLazyQuery,
  useLedgerAccountExistsByIdLazyQuery,
} from "@/lib/graphql/generated"

gql`
  query LedgerAccountExistsByCode($code: String!) {
    ledgerAccountByCode(code: $code) {
      id
    }
  }

  query LedgerAccountExistsById($id: UUID!) {
    ledgerAccount(id: $id) {
      id
    }
  }
`

export default function LedgerAccount() {
  const router = useRouter()
  const t = useTranslations("ChartOfAccountsLedgerAccount")
  const [searchTerm, setSearchTerm] = useState("")
  const [isSearching, setIsSearching] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const [checkByCode] = useLedgerAccountExistsByCodeLazyQuery()
  const [checkById] = useLedgerAccountExistsByIdLazyQuery()

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault()

    const trimmedTerm = searchTerm.trim()
    if (!trimmedTerm) return

    setIsSearching(true)
    setError(null)

    try {
      const isId = isUUID(trimmedTerm)
      let accountExists = false

      if (isId) {
        const { data } = await checkById({ variables: { id: trimmedTerm } })
        accountExists = !!data?.ledgerAccount?.id
      } else {
        const { data } = await checkByCode({ variables: { code: trimmedTerm } })
        accountExists = !!data?.ledgerAccountByCode?.id
      }

      if (accountExists) {
        router.push(`/ledger-account/${encodeURIComponent(trimmedTerm)}`)
      } else {
        setError(t("search.notFound"))
      }
    } catch (error) {
      setError(t("search.searchError"))
    } finally {
      setIsSearching(false)
    }
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchTerm(e.target.value)
    if (error) setError(null)
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault()
      handleSearch(e as React.FormEvent)
    }
  }

  const isValidInput = searchTerm.trim().length > 0

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("search.title")}</CardTitle>
        <CardDescription>{t("search.description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSearch} className="space-y-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              type="text"
              placeholder={t("search.placeholder")}
              value={searchTerm}
              onChange={handleInputChange}
              onKeyDown={handleKeyDown}
              className="pl-10"
              disabled={isSearching}
            />
          </div>
          <div className="flex items-center justify-between gap-4">
            <div className="flex-1">
              {error && (
                <div className="text-destructive text-sm bg-destructive/10 p-2 rounded-md">
                  {error}
                </div>
              )}
            </div>
            <Button type="submit" loading={isSearching} disabled={!isValidInput}>
              {t("search.searchButton")}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}
