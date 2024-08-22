"use client"
import { useState } from "react"

import { IoCaretDownSharp, IoCaretForwardSharp } from "react-icons/io5"

import { AccountSetSubAccount, usePnlAccountSetQuery } from "@/lib/graphql/generated"
import Balance, { Currency } from "@/components/balance/balance"
import { TableCell, TableRow } from "@/components/primitive/table"
import { DateRange } from "@/components/date-range-picker"

export const Account = ({
  account,
  currency,
  depth = 0,
  layer,
  transactionType,
  dateRange,
}: {
  account: AccountSetSubAccount
  currency: Currency
  depth?: number
  layer: Layers
  transactionType: TransactionType
  dateRange: DateRange
}) => {
  const [showingSubAccounts, setShowingSubAccounts] = useState(false)
  const hasSubAccounts = account.__typename === "AccountSet" && account.hasSubAccounts

  return (
    <>
      <TableRow
        key={account.id}
        className={hasSubAccounts ? "cursor-pointer" : ""}
        onClick={() => setShowingSubAccounts((toggle) => !toggle)}
      >
        <TableCell className="flex items-center">
          {Array.from({ length: depth }).map((_, i) => (
            <div key={i} className="w-8" />
          ))}
          <div className="w-8">
            {hasSubAccounts &&
              (showingSubAccounts ? <IoCaretDownSharp /> : <IoCaretForwardSharp />)}
          </div>
          <div>{account.name}</div>
        </TableCell>
        <TableCell>
          <Balance
            align="end"
            currency={currency}
            amount={account.amounts[currency].closingBalance[layer][transactionType]}
          />
        </TableCell>
      </TableRow>

      {hasSubAccounts && showingSubAccounts && (
        <SubAccountsForAccountSet
          currency={currency}
          account={account}
          depth={depth}
          layer={layer}
          transactionType={transactionType}
          dateRange={dateRange}
        />
      )}
    </>
  )
}

const SubAccountsForAccountSet = ({
  account,
  depth = 0,
  currency,
  layer,
  transactionType,
  dateRange,
}: {
  account: AccountSetSubAccount
  depth?: number
  currency: Currency
  layer: Layers
  transactionType: TransactionType
  dateRange: DateRange
}) => {
  const { data, fetchMore } = usePnlAccountSetQuery({
    variables: {
      accountSetId: account.id,
      first: 10,
      from: dateRange.from,
      until: dateRange.until,
    },
    fetchPolicy: "cache-and-network",
  })

  const hasMoreSubAccounts = data?.accountSet?.subAccounts.pageInfo.hasNextPage
  const subAccounts = data?.accountSet?.subAccounts.edges

  return (
    <>
      {subAccounts?.map((subAccount) => (
        <Account
          currency={currency}
          key={subAccount.node.id}
          account={subAccount.node}
          depth={depth + 1}
          layer={layer}
          transactionType={transactionType}
          dateRange={dateRange}
        />
      ))}
      {hasMoreSubAccounts && subAccounts && (
        <TableRow>
          <TableCell
            className="flex items-center cursor-pointer"
            onClick={() =>
              fetchMore({
                variables: {
                  after: subAccounts[subAccounts.length - 1].cursor,
                },
              })
            }
          >
            {Array.from({ length: depth + 1 }).map((_, i) => (
              <div key={i} className="w-8" />
            ))}
            <div className="w-8" />
            <div className="font-thin italic">show more...</div>
          </TableCell>
          <TableCell></TableCell>
        </TableRow>
      )}
    </>
  )
}
