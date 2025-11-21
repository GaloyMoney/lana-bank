"use client"

import React from "react"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { LedgerTransactionFieldsFragment } from "@/lib/graphql/generated"
import CardWrapper from "@/components/card-wrapper"
import DataTable, { Column } from "@/components/data-table"

interface LedgerTransactionsProps {
  ledgerTransactions: LedgerTransactionFieldsFragment[]
}

const LedgerTransactions: React.FC<LedgerTransactionsProps> = ({
  ledgerTransactions,
}) => {
  const t = useTranslations("LedgerTransactions")
  const columns: Column<LedgerTransactionFieldsFragment>[] = [
    {
      key: "description",
      header: t("table.headers.description"),
    },
    {
      key: "effective",
      header: t("table.headers.effectiveDate"),
      render: (value) => <DateWithTooltip value={value} />,
    },
  ]
  return (
    <CardWrapper title={t("title")}>
      <DataTable
        data={[...ledgerTransactions].reverse()}
        columns={columns}
        emptyMessage={t("noTransactions")}
        navigateTo={(transaction) =>
          `/ledger-transactions/${transaction.ledgerTransactionId}`
        }
      />
    </CardWrapper>
  )
}

export default LedgerTransactions
