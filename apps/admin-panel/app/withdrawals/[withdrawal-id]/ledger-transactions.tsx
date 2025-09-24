"use client"

import React, { useMemo } from "react"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { GetWithdrawalDetailsQuery } from "@/lib/graphql/generated"
import CardWrapper from "@/components/card-wrapper"
import DataTable, { Column } from "@/components/data-table"

type LedgerTransaction = NonNullable<
  NonNullable<GetWithdrawalDetailsQuery["withdrawalByPublicId"]>["ledgerTransactions"]
>[0]

interface LedgerTransactionsProps {
  ledgerTransactions: LedgerTransaction[]
}

const LedgerTransactions: React.FC<LedgerTransactionsProps> = ({
  ledgerTransactions,
}) => {
  const t = useTranslations("Withdrawals.WithdrawDetails.LedgerTransactions")

  const sortedTransactions = useMemo(() => {
    return [...ledgerTransactions].sort(
      (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    )
  }, [ledgerTransactions])

  const columns: Column<LedgerTransaction>[] = [
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
        data={sortedTransactions}
        columns={columns}
        emptyMessage={t("noTransactions")}
        navigateTo={(transaction) =>
          `/ledger-transaction/${transaction.ledgerTransactionId}`
        }
      />
    </CardWrapper>
  )
}

export default LedgerTransactions
