"use client"

import { gql } from "@apollo/client"
import { useCallback, useMemo, useState } from "react"
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

import { Account } from "./account"
import { AsOfDateSelector, getInitialAsOfDate } from "./as-of-date-selector"

import Balance, { Currency } from "@/components/balance/balance"
import {
  CurrencySelection,
  LayerSelection,
  ReportLayer,
} from "@/components/report-filters/selectors"
import { BalanceSheetQuery, useBalanceSheetQuery } from "@/lib/graphql/generated"

gql`
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
      categories {
        balanceSheetAccountId
        ledgerAccountId
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
        children {
<<<<<<< HEAD
          balanceSheetAccountId
=======
          balanceSheetAccountSetId
          ledgerAccountId
>>>>>>> db69a3b6b (fix(admin-panel): use ledgerAccountId in balance sheet links)
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
      }
    }
  }
`

type CategoryBalance = NonNullable<BalanceSheetQuery["balanceSheet"]>["assetsBalance"]

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

  if (error) return <div className="text-destructive">{error.message}</div>

  const assets = data?.categories?.filter((cat) => cat.name === "Assets")
  const liabilities = data?.categories?.filter((cat) => cat.name === "Liabilities")
  const equity = data?.categories?.filter((cat) => cat.name === "Equity")
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
            <Label>{t("asOf")}</Label>
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
  categories: NonNullable<BalanceSheetQuery["balanceSheet"]>["categories"]
  currency: Currency
  layer: ReportLayer
  total: number
}

function BalanceSheetColumn({
  title,
  categories,
  currency,
  layer,
  total,
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
  category: NonNullable<BalanceSheetQuery["balanceSheet"]>["categories"][0]
  currency: Currency
  layer: ReportLayer
}

function CategoryRow({ category, currency, layer }: CategoryRowProps) {
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
          account={child}
          currency={currency}
          layer={layer}
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
