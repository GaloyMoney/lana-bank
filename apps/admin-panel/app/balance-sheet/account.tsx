"use client"
import { TableCell, TableRow } from "@lana/web/ui/table"

import { useRouter } from "next/navigation"

import Balance, { Currency } from "@/components/balance/balance"
import { BalanceSheetQuery } from "@/lib/graphql/generated"

type BalanceSheetBalance = NonNullable<
  BalanceSheetQuery["balanceSheet"]
>["rows"][number]["balance"]

export interface BalanceSheetAccountNode {
  balanceSheetAccountId: string
  ledgerAccountId: string
  code?: string | null
  name: string
  balance: BalanceSheetBalance
  children?: BalanceSheetAccountNode[]
}

interface AccountProps {
  account: BalanceSheetAccountNode
  currency: Currency
  depth?: number
  layer: BalanceSheetLayers
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

  const balance = account.balance[currency][layer].net
  const hasChildren = (account.children?.length ?? 0) > 0
  const isCollapsed = collapsedAccountIds.has(account.balanceSheetAccountId)

  const handleRowClick = () => {
    router.push(`/ledger-accounts/${account.code || account.ledgerAccountId}`)
  }

  return (
    <>
      <TableRow
        data-testid={`account-${account.balanceSheetAccountId}`}
        key={account.balanceSheetAccountId}
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
                data-testid={`toggle-${account.balanceSheetAccountId}`}
                className="inline-flex h-5 w-5 items-center justify-center rounded-sm text-muted-foreground hover:bg-muted"
                onClick={(e) => {
                  e.stopPropagation()
                  onToggleCollapsed(account.balanceSheetAccountId)
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
          <Balance
            align="end"
            className="font-semibold"
            currency={currency}
            amount={balance as CurrencyType}
          />
        </TableCell>
      </TableRow>
      {!isCollapsed &&
        account.children?.map((child) => (
          <Account
            key={child.balanceSheetAccountId}
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
