"use client"
import { TableCell, TableRow } from "@lana/web/ui/table"

import { useRouter } from "next/navigation"

import Balance, { Currency } from "@/components/balance/balance"
import { ProfitAndLossStatementQuery } from "@/lib/graphql/generated"

type BalanceRangeType = NonNullable<
  ProfitAndLossStatementQuery["profitAndLossStatement"]
>["rows"][number]["balanceRange"]

export interface ProfitAndLossAccountNode {
  profitAndLossAccountId: string
  ledgerAccountId: string
  code?: string | null
  name: string
  balanceRange: BalanceRangeType
  children?: ProfitAndLossAccountNode[]
}

interface AccountProps {
  account: ProfitAndLossAccountNode
  currency: Currency
  depth?: number
  layer: PnlLayers
  collapsedAccountIds: Set<string>
  onToggleCollapsed: (accountId: string) => void
}

export const Account = ({
  account,
  currency,
  depth = 0,
  layer,
  collapsedAccountIds,
  onToggleCollapsed,
}: AccountProps) => {
  const router = useRouter()

  let accountPeriod: number | undefined
  const hasChildren = (account.children?.length ?? 0) > 0
  const isCollapsed = collapsedAccountIds.has(account.profitAndLossAccountId)

  if (account.balanceRange.__typename === "UsdLedgerAccountBalanceRange") {
    accountPeriod = account.balanceRange.usdDiff[layer].net
  } else if (account.balanceRange.__typename === "BtcLedgerAccountBalanceRange") {
    accountPeriod = account.balanceRange.btcDiff[layer].net
  }

  const handleRowClick = () => {
    router.push(`/ledger-accounts/${account.code || account.ledgerAccountId}`)
  }

  return (
    <>
      <TableRow
        data-testid={`account-${account.profitAndLossAccountId}`}
        className="cursor-pointer hover:bg-muted/50"
        onClick={handleRowClick}
      >
        <TableCell className="flex items-center">
          {Array.from({ length: depth }).map((_, i) => (
            <div key={i} className="w-8" />
          ))}
          <div className="flex w-8 justify-center">
            {hasChildren ? (
              <button
                type="button"
                data-testid={`toggle-${account.profitAndLossAccountId}`}
                className="inline-flex h-5 w-5 items-center justify-center rounded-sm text-muted-foreground hover:bg-muted"
                onClick={(e) => {
                  e.stopPropagation()
                  onToggleCollapsed(account.profitAndLossAccountId)
                }}
                aria-label={isCollapsed ? "Expand account" : "Collapse account"}
              >
                {isCollapsed ? "▸" : "▾"}
              </button>
            ) : null}
          </div>
          <div>{account.name}</div>
        </TableCell>
        <TableCell>
          <Balance align="end" currency={currency} amount={accountPeriod as CurrencyType} />
        </TableCell>
      </TableRow>
      {!isCollapsed &&
        account.children?.map((child) => (
          <Account
            key={child.profitAndLossAccountId}
            account={child}
            currency={currency}
            depth={depth + 1}
            layer={layer}
            collapsedAccountIds={collapsedAccountIds}
            onToggleCollapsed={onToggleCollapsed}
          />
        ))}
    </>
  )
}
