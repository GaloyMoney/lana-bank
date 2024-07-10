"use client"

import React from "react"
import { IoCaretDownSharp, IoCaretForwardSharp } from "react-icons/io5"

import {
  ChartOfAccountsSubAccount,
  useChartOfAccountAccountSetQuery,
} from "@/lib/graphql/generated"
import { TableCell, TableRow } from "@/components/primitive/table"

type AccountProps = {
  depth?: number
  account: ChartOfAccountsSubAccount
}

const SubAccountsForAccountSet: React.FC<AccountProps> = ({ account, depth = 0 }) => {
  const { data } = useChartOfAccountAccountSetQuery({
    variables: {
      id: account.id,
    },
  })

  const subAccounts = data?.chartOfAccountsAccountSet?.subAccounts

  return subAccounts?.map((subAccount) => (
    <Account key={subAccount.id} account={subAccount} depth={depth + 1} />
  ))
}

export const Account: React.FC<AccountProps> = ({ account, depth = 0 }) => {
  const [showingSubAccounts, setShowingSubAccounts] = React.useState(false)
  const hasSubAccounts =
    account.__typename === "AccountSetDetails" && account.hasSubAccounts

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
      </TableRow>

      {hasSubAccounts && showingSubAccounts && (
        <SubAccountsForAccountSet account={account} depth={depth} />
      )}
    </>
  )
}
