"use client"

import { gql } from "@apollo/client"
import { useCallback, useEffect, useMemo, useState } from "react"
import { useTranslations } from "next-intl"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Label } from "@lana/web/ui/label"
import { Separator } from "@lana/web/ui/separator"
import { Skeleton } from "@lana/web/ui/skeleton"
import { Table, TableBody, TableCell, TableRow } from "@lana/web/ui/table"

import { Account, BalanceSheetAccountNode } from "./account"
import { AsOfDateSelector, getInitialAsOfDate } from "./as-of-date-selector"

import Balance, { Currency } from "@/components/balance/balance"
import {
  CurrencySelection,
  LayerSelection,
  ReportLayer,
} from "@/components/report-filters/selectors"
import { BalanceSheetQuery, useBalanceSheetQuery } from "@/lib/graphql/generated"

gql`
  fragment BalanceSheetRowFields on BalanceSheetRow {
    balanceSheetAccountId
    parentBalanceSheetAccountId
    ledgerAccountId
    category
    depth
    name
    code
    balance {
      usd {
        settled {
          net
        }
        pending {
          net
        }
      }
      btc {
        settled {
          net
        }
        pending {
          net
        }
      }
    }
  }

  query BalanceSheet($asOf: Date!) {
    balanceSheet(asOf: $asOf) {
      name
      assetsBalance {
        usd {
          settled {
            net
          }
          pending {
            net
          }
        }
        btc {
          settled {
            net
          }
          pending {
            net
          }
        }
      }
      liabilitiesBalance {
        usd {
          settled {
            net
          }
          pending {
            net
          }
        }
        btc {
          settled {
            net
          }
          pending {
            net
          }
        }
      }
      equityBalance {
        usd {
          settled {
            net
          }
          pending {
            net
          }
        }
        btc {
          settled {
            net
          }
          pending {
            net
          }
        }
      }
      rows {
        ...BalanceSheetRowFields
      }
    }
  }
`

type CategoryBalance = NonNullable<BalanceSheetQuery["balanceSheet"]>["assetsBalance"]
type BalanceSheetRow = NonNullable<BalanceSheetQuery["balanceSheet"]>["rows"][number]

export default function BalanceSheetPage() {
  const initialAsOf = useMemo(() => getInitialAsOfDate(), [])
  const [asOf, setAsOf] = useState(initialAsOf)
  const handleDateChange = useCallback((newAsOf: string) => {
    setAsOf(newAsOf)
  }, [])

  const { data, loading, error } = useBalanceSheetQuery({
    variables: { asOf },
    fetchPolicy: "cache-and-network",
  })

  return (
    <BalanceSheetView
      data={data?.balanceSheet}
      loading={loading && !data}
      error={error}
      asOf={asOf}
      setAsOf={handleDateChange}
    />
  )
}

interface BalanceSheetViewProps {
  data?: BalanceSheetQuery["balanceSheet"]
  loading: boolean
  error: Error | undefined
  asOf: string
  setAsOf: (asOf: string) => void
}

const BalanceSheetView = ({
  data,
  loading,
  error,
  asOf,
  setAsOf,
}: BalanceSheetViewProps) => {
  const t = useTranslations("BalanceSheet")
  const [currency, setCurrency] = useState<Currency>("usd")
  const [layer, setLayer] = useState<ReportLayer>("settled")
  const [collapsedAccountIds, setCollapsedAccountIds] = useState<Set<string>>(new Set())
  const toggleCollapsedAccount = useCallback((accountId: string) => {
    setCollapsedAccountIds((prev) => {
      const next = new Set(prev)
      if (next.has(accountId)) {
        next.delete(accountId)
      } else {
        next.add(accountId)
      }
      return next
    })
  }, [])
  const categories = useMemo(() => buildBalanceSheetTree(data?.rows ?? []), [data?.rows])

  useEffect(() => {
    if (categories.length === 0) return
    setCollapsedAccountIds((prev) => {
      if (prev.size > 0) return prev
      return collectCollapsedBalanceSheetAccountIds(categories)
    })
  }, [categories])

  if (error) return <div className="text-destructive">{error.message}</div>

  const assets = categories.filter((cat) => cat.name === "Assets")
  const liabilities = categories.filter((cat) => cat.name === "Liabilities")
  const equity = categories.filter((cat) => cat.name === "Equity")
  const liabilitiesAndEquity = [...(liabilities || []), ...(equity || [])]

  const assetsTotal = getBalanceNet(data?.assetsBalance, currency, layer)
  const liabilitiesAndEquityTotal =
    getBalanceNet(data?.liabilitiesBalance, currency, layer) +
    getBalanceNet(data?.equityBalance, currency, layer)

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="mb-2 flex w-fit flex-wrap items-center gap-2 rounded-md">
          <div>
            <Label>{t("date")}</Label>
            <AsOfDateSelector asOf={asOf} onDateChange={setAsOf} />
          </div>
          <div className="flex h-14 items-center">
            <Separator orientation="vertical" />
          </div>
          <div className="flex flex-col gap-2">
            <CurrencySelection currency={currency} setCurrency={setCurrency} />
          </div>
          <div className="flex h-14 items-center">
            <Separator orientation="vertical" />
          </div>
          <div className="flex flex-col gap-2">
            <LayerSelection layer={layer} setLayer={setLayer} />
          </div>
        </div>

        {loading || !data ? (
          <Skeleton className="h-96 w-full" />
        ) : (
          <div className="flex justify-between rounded-md border">
            {assets && assets.length > 0 && (
              <BalanceSheetColumn
                title={t("columns.assets")}
                categories={assets}
                currency={currency}
                layer={layer}
                total={assetsTotal}
                collapsedAccountIds={collapsedAccountIds}
                onToggleCollapsed={toggleCollapsedAccount}
              />
            )}
            <div className="min-h-full w-px bg-border" />
            {liabilitiesAndEquity.length > 0 && (
              <BalanceSheetColumn
                title={t("columns.liabilitiesAndEquity")}
                categories={liabilitiesAndEquity}
                currency={currency}
                layer={layer}
                total={liabilitiesAndEquityTotal}
                collapsedAccountIds={collapsedAccountIds}
                onToggleCollapsed={toggleCollapsedAccount}
              />
            )}
          </div>
        )}
      </CardContent>
    </Card>
  )
}

interface BalanceSheetColumnProps {
  title: string
  categories: BalanceSheetAccountNode[]
  currency: Currency
  layer: ReportLayer
  total: number
  collapsedAccountIds: Set<string>
  onToggleCollapsed: (accountId: string) => void
}

function BalanceSheetColumn({
  title,
  categories,
  currency,
  layer,
  total,
  collapsedAccountIds,
  onToggleCollapsed,
}: BalanceSheetColumnProps) {
  return (
    <div className="flex w-1/2 grow flex-col justify-between">
      <Table>
        <TableBody>
          {categories.map((category) => (
            <CategoryRow
              key={category.balanceSheetAccountId}
              category={category}
              currency={currency}
              layer={layer}
              collapsedAccountIds={collapsedAccountIds}
              onToggleCollapsed={onToggleCollapsed}
            />
          ))}
        </TableBody>
      </Table>
      <Table>
        <TableBody>
          <TableRow className="bg-secondary">
            <TableCell className="font-bold uppercase">{title}</TableCell>
            <TableCell className="flex flex-col items-end gap-2 text-right font-semibold">
              <Balance
                align="end"
                className="font-semibold"
                currency={currency}
                amount={total as CurrencyType}
              />
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  )
}

interface CategoryRowProps {
  category: BalanceSheetAccountNode
  currency: Currency
  layer: ReportLayer
  collapsedAccountIds: Set<string>
  onToggleCollapsed: (accountId: string) => void
}

function CategoryRow({
  category,
  currency,
  layer,
  collapsedAccountIds,
  onToggleCollapsed,
}: CategoryRowProps) {
  const t = useTranslations("BalanceSheet")
  const categoryBalance = getBalanceNet(category.balance, currency, layer)

  return (
    <>
      <TableRow className="bg-secondary">
        <TableCell
          className="flex items-center gap-2 font-semibold uppercase text-primary"
          data-testid={`category-name-${category.name.toLowerCase()}`}
        >
          {t(`categories.${category.name.replace(/\s+/g, "")}`)}
        </TableCell>
        <TableCell className="w-48" />
      </TableRow>
      {category.children?.map((child) => (
        <Account
          key={child.balanceSheetAccountId}
          account={child as BalanceSheetAccountNode}
          currency={currency}
          layer={layer}
          collapsedAccountIds={collapsedAccountIds}
          onToggleCollapsed={onToggleCollapsed}
        />
      ))}
      <TableRow>
        <TableCell className="flex items-center gap-2 text-xs font-semibold uppercase text-textColor-secondary">
          <div className="w-6" />
          {t("total")}
        </TableCell>
        <TableCell>
          <Balance
            align="end"
            className="font-semibold"
            currency={currency}
            amount={categoryBalance as CurrencyType}
          />
        </TableCell>
      </TableRow>
    </>
  )
}

function getBalanceNet(
  balance: CategoryBalance | undefined,
  currency: Currency,
  layer: ReportLayer,
): number {
  if (!balance) return 0
  return balance[currency][layer].net
}

function buildBalanceSheetTree(rows: BalanceSheetRow[]): BalanceSheetAccountNode[] {
  const nodesById = new Map<string, BalanceSheetAccountNode>()
  for (const row of rows) {
    nodesById.set(row.balanceSheetAccountId, {
      balanceSheetAccountId: row.balanceSheetAccountId,
      ledgerAccountId: row.ledgerAccountId,
      code: row.code,
      name: row.name,
      balance: row.balance,
      children: [],
    })
  }

  const roots: BalanceSheetAccountNode[] = []
  for (const row of rows) {
    const node = nodesById.get(row.balanceSheetAccountId)
    if (!node) continue

    if (!row.parentBalanceSheetAccountId) {
      roots.push(node)
      continue
    }

    const parent = nodesById.get(row.parentBalanceSheetAccountId)
    if (!parent) {
      roots.push(node)
      continue
    }
    parent.children = [...(parent.children ?? []), node]
  }

  return roots
}

function collectCollapsedBalanceSheetAccountIds(
  nodes: BalanceSheetAccountNode[],
): Set<string> {
  const ids = new Set<string>()

  const walk = (node: BalanceSheetAccountNode) => {
    if ((node.children?.length ?? 0) > 0) {
      ids.add(node.balanceSheetAccountId)
      node.children?.forEach(walk)
    }
  }

  nodes.forEach(walk)
  return ids
}
