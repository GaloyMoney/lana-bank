"use client"
import { gql } from "@apollo/client"
import { useState } from "react"

import { Account } from "./account"

import {
  ProfitAndLossStatementQuery,
  StatementCategoryWithBalance,
  useProfitAndLossStatementQuery,
} from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import {
  Table,
  TableBody,
  TableCell,
  TableFooter,
  TableRow,
} from "@/components/primitive/table"

import { PageHeading } from "@/components/page-heading"
import { CurrencyLayerSelection } from "@/components/financial/currency-layer-selection"

gql`
  query ProfitAndLossStatement($from: Timestamp!, $until: Timestamp) {
    profitAndLossStatement(from: $from, until: $until) {
      name
      balance {
        ...balancesByCurrency
      }
      categories {
        name
        balance {
          ...balancesByCurrency
        }
        accounts {
          ... on AccountWithBalance {
            __typename
            id
            name
            balance {
              ...balancesByCurrency
            }
          }
          ... on AccountSetWithBalance {
            __typename
            id
            name
            hasSubAccounts
            balance {
              ...balancesByCurrency
            }
          }
        }
      }
    }
  }
`

const BALANCE_FOR_CATEGORY: {
  [key: string]: { TransactionType: TransactionType }
} = {
  Revenue: { TransactionType: "netCredit" },
  Expenses: { TransactionType: "netDebit" },
}

export default function ProfitAndLossStatementPage() {
  const {
    data: ProfitAndLossStatementData,
    loading: ProfitAndLossStatementLoading,
    error: ProfitAndLossStatementError,
  } = useProfitAndLossStatementQuery({
    variables: {
      from: new Date(Date.now()),
    },
  })

  return (
    <ProfitAndLossStatement
      data={ProfitAndLossStatementData?.profitAndLossStatement}
      loading={ProfitAndLossStatementLoading}
      error={ProfitAndLossStatementError}
    />
  )
}

const ProfitAndLossStatement = ({
  data,
  loading,
  error,
}: {
  data: ProfitAndLossStatementQuery["profitAndLossStatement"]
  loading: boolean
  error: Error | undefined
}) => {
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<Layers>("all")

  const balance = data?.balance
  const categories = data?.categories

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading) return <div>Loading...</div>
  if (!balance) return <div>No data</div>

  return (
    <main>
      <div>
        <PageHeading>Profit and Loss</PageHeading>
        <CurrencyLayerSelection
          currency={currency}
          setCurrency={setCurrency}
          layer={layer}
          setLayer={setLayer}
        />
      </div>
      <Table className="mt-6">
        <TableBody>
          {categories?.map((category) => {
            return (
              <CategoryRow
                key={category.name}
                category={category}
                currency={currency}
                layer={layer}
                transactionType={
                  BALANCE_FOR_CATEGORY[category.name].TransactionType || "netCredit"
                }
              />
            )
          })}
        </TableBody>
        <TableFooter>
          <TableRow>
            <TableCell className="uppercase pr-10 text-textColor-secondary">
              NET
            </TableCell>
            <TableCell className="w-48">
              <Balance
                align="end"
                currency={currency}
                amount={balance[currency].end[layer].netCredit}
              />
            </TableCell>
          </TableRow>
        </TableFooter>
      </Table>
    </main>
  )
}

const CategoryRow = ({
  category,
  currency,
  layer,
  transactionType,
}: {
  category: StatementCategoryWithBalance
  currency: Currency
  layer: Layers
  transactionType: TransactionType
}) => {
  console.log(category.name, transactionType)

  return (
    <>
      <TableRow>
        <TableCell className="flex items-center gap-2 text-primary font-semibold uppercase">
          {category.name}
        </TableCell>
        <TableCell className="w-48">
          <Balance
            align="end"
            currency={currency}
            amount={category.balance[currency].end[layer][transactionType]}
          />
        </TableCell>
      </TableRow>
      {category.accounts.map((account) => (
        <Account
          key={account.id}
          account={account}
          currency={currency}
          layer={layer}
          transactionType={transactionType}
        />
      ))}
    </>
  )
}
