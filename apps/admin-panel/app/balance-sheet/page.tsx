"use client"
import { gql } from "@apollo/client"
import { useState } from "react"

import { Account } from "./account"

import {
  BalanceSheetQuery,
  StatementCategory,
  useBalanceSheetQuery,
} from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import { Table, TableBody, TableCell, TableRow } from "@/components/primitive/table"

import { PageHeading } from "@/components/page-heading"
import { CurrencyLayerSelection } from "@/components/financial/currency-layer-selection"

gql`
  query BalanceSheet($from: Timestamp!, $until: Timestamp) {
    balanceSheet(from: $from, until: $until) {
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
          ... on Account {
            __typename
            id
            name
            balance {
              ...balancesByCurrency
            }
          }
          ... on AccountSet {
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

const BALANCE_FOR_CATEGORY: Record<string, { TransactionType: TransactionType }> = {
  Liabilities: { TransactionType: "netCredit" },
  Equity: { TransactionType: "netCredit" },
  Assets: { TransactionType: "netDebit" },
}

export default function BalanceSheetPage() {
  const { data, loading, error } = useBalanceSheetQuery({
    variables: {
      from: new Date(Date.now()),
    },
  })
  return <BalanceSheet data={data?.balanceSheet} loading={loading} error={error} />
}

const BalanceSheet = ({
  data,
  loading,
  error,
}: {
  data: BalanceSheetQuery["balanceSheet"]
  loading: boolean
  error: Error | undefined
}) => {
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<Layers>("all")

  if (error) return <div className="text-destructive">{error.message}</div>
  if (loading) return <div>Loading...</div>
  if (!data?.balance) return <div>No data</div>

  const assets = data.categories?.filter((category) => category.name === "Assets")
  const liabilitiesAndEquity = data.categories?.filter(
    (category) => category.name === "Liabilities" || category.name === "Equity",
  )

  const totalLiabilitiesAndEquity = calculateTotalLiabilitiesAndEquity(
    liabilitiesAndEquity,
    currency,
    layer,
  )

  return (
    <main>
      <BalanceSheetHeader
        currency={currency}
        setCurrency={setCurrency}
        layer={layer}
        setLayer={setLayer}
      />
      <div className="flex gap-4 justify-between mt-6">
        {assets && assets.length > 0 && (
          <BalanceSheetColumn
            title="Total Assets"
            categories={assets}
            currency={currency}
            layer={layer}
            total={
              assets[0].balance[currency].closingBalance[layer][
                BALANCE_FOR_CATEGORY["Assets"].TransactionType
              ]
            }
          />
        )}
        <div className="w-1 min-h-full bg-secondary-foreground"></div>
        {liabilitiesAndEquity && liabilitiesAndEquity.length > 0 && (
          <BalanceSheetColumn
            title="Total Liabilities & Equity"
            categories={liabilitiesAndEquity}
            currency={currency}
            layer={layer}
            total={totalLiabilitiesAndEquity}
          />
        )}
      </div>
    </main>
  )
}

function BalanceSheetHeader({
  currency,
  setCurrency,
  layer,
  setLayer,
}: {
  currency: Currency
  setCurrency: (currency: Currency) => void
  layer: Layers
  setLayer: (layer: Layers) => void
}) {
  return (
    <div>
      <PageHeading>Balance Sheet</PageHeading>
      <CurrencyLayerSelection
        currency={currency}
        setCurrency={setCurrency}
        layer={layer}
        setLayer={setLayer}
      />
    </div>
  )
}

function BalanceSheetColumn({
  title,
  categories,
  currency,
  layer,
  total,
}: {
  title: string
  categories: StatementCategory[]
  currency: Currency
  layer: Layers
  total: number
}) {
  return (
    <div className="flex-grow flex flex-col justify-between w-1/2">
      <Table>
        <TableBody>
          {categories.map((category) => (
            <CategoryRow
              key={category.name}
              category={category}
              currency={currency}
              layer={layer}
              transactionType={BALANCE_FOR_CATEGORY[category.name].TransactionType}
            />
          ))}
        </TableBody>
      </Table>
      <Table>
        <TableBody>
          <TableRow className="bg-secondary-foreground">
            <TableCell className="uppercase font-bold text-textColor-secondary">
              {title}
            </TableCell>
            <TableCell className="flex flex-col gap-2 items-end text-right">
              <Balance align="end" currency={currency} amount={total} />
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  )
}

function CategoryRow({
  category,
  currency,
  layer,
  transactionType,
}: {
  category: StatementCategory
  currency: Currency
  layer: Layers
  transactionType: TransactionType
}) {
  return (
    <>
      <TableRow className="bg-secondary-foreground">
        <TableCell className="flex items-center gap-2 text-primary font-semibold uppercase ">
          {category.name}
        </TableCell>
        <TableCell className="w-48"></TableCell>
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
      {category.name !== "Assets" && (
        <TableRow>
          <TableCell className="flex items-center gap-2 text-textColor-secondary font-semibold uppercase text-xs">
            <div className="w-6" />
            Total
          </TableCell>
          <TableCell>
            <Balance
              align="end"
              currency={currency}
              amount={category.balance[currency].closingBalance[layer][transactionType]}
            />
          </TableCell>
        </TableRow>
      )}
    </>
  )
}

function calculateTotalLiabilitiesAndEquity(
  categories: StatementCategory[] | undefined,
  currency: Currency,
  layer: Layers,
): number {
  return (
    categories?.reduce(
      (acc, category) =>
        acc +
        category.balance[currency].closingBalance[layer][
          BALANCE_FOR_CATEGORY[category.name].TransactionType
        ],
      0,
    ) || 0
  )
}
