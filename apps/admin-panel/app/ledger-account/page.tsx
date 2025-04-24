"use client"

import { gql } from "@apollo/client"
import React, { ChangeEvent, useState } from "react"

import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"
import { LoadingSpinner } from "@lana/web/ui/loading-spinner"
import { useTranslations } from "next-intl"
import Link from "next/link"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { isUUID, debounce } from "@/lib/utils"
import {
  LedgerAccountNameByIdQuery,
  LedgerAccountNameByCodeQuery,
  useLedgerAccountNameByCodeLazyQuery,
  useLedgerAccountNameByIdLazyQuery,
} from "@/lib/graphql/generated"

gql`
  fragment LedgerAccountName on LedgerAccount {
    id
    name
    code
    ancestors {
      id
      name
      code
    }
  }
  query LedgerAccountNameByCode($code: String!) {
    ledgerAccountByCode(code: $code) {
      ...LedgerAccountName
    }
  }

  query LedgerAccountNameById($id: UUID!) {
    ledgerAccount(id: $id) {
      ...LedgerAccountName
    }
  }
`

export default function LedgerAccount() {
  const t = useTranslations("LedgerAccounts")
  const [accountData, setAccountData] = useState<
    | LedgerAccountNameByCodeQuery["ledgerAccountByCode"]
    | LedgerAccountNameByIdQuery["ledgerAccount"]
  >()
  const [showErrorFetching, setShowErrorFetching] = useState(false)
  const [loading, setLoading] = useState(false)

  const [ledgerAccountByCodeQuery] = useLedgerAccountNameByCodeLazyQuery()
  const [ledgerAccountByIdQuery] = useLedgerAccountNameByIdLazyQuery()

  const handleLedgerAccountSearchChange = async (e: ChangeEvent<HTMLInputElement>) => {
    setLoading(true)
    setShowErrorFetching(false)
    const { value } = e.target

    let accountQuery
    if (isUUID(value)) {
      accountQuery = await ledgerAccountByIdQuery({
        variables: { id: value },
      })
    } else {
      accountQuery = await ledgerAccountByCodeQuery({
        variables: { code: value },
      })
    }
    const { data, errors } = accountQuery

    if (errors?.length || !data) {
      setAccountData(null)
      setShowErrorFetching(true)
    } else {
      if ("ledgerAccount" in data && data.ledgerAccount !== null) {
        setAccountData(data.ledgerAccount)
      } else if ("ledgerAccountByCode" in data && data.ledgerAccountByCode !== null) {
        setAccountData(data.ledgerAccountByCode)
      } else {
        setAccountData(null)
        setShowErrorFetching(true)
      }
    }
    setLoading(false)
  }

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <Label htmlFor="ledger-accounts">{t("form.labels.accountSearch")}</Label>
          <div className="flex flex-wrap md:flex-nowrap">
            <Input
              id="ledger-accounts"
              type="string"
              className="mr-2"
              onChange={debounce((e) => handleLedgerAccountSearchChange(e), 150)}
              required
              placeholder={t("form.placeholders.accountSearch")}
              data-testid="search-ledger-accounts"
            />
          </div>
          {showErrorFetching && (
            <p className="text-destructive text-sm">{t("form.errors.accountSearch")}</p>
          )}
        </CardContent>
      </Card>
      {loading && (
        <Card className="mt-2 flex items-center justify-center p-4">
          <LoadingSpinner />
        </Card>
      )}
      {accountData && accountData.id && (
        <Link href={`/ledger-account/${accountData.id}`}>
          <Card className="mt-4 flex items-center  justify-between">
            <CardHeader>
              <CardTitle>{t("searchResult.title")}</CardTitle>
              <CardDescription>{accountData?.name || accountData.code}</CardDescription>
              {accountData.id}
            </CardHeader>
          </Card>
        </Link>
      )}
    </>
  )
}
