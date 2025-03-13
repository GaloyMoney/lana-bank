"use client"

import React, { useState, useCallback, MouseEventHandler } from "react"
import { ApolloError, gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { IoCaretDownSharp, IoCaretForwardSharp } from "react-icons/io5"

import { Skeleton } from "@lana/web/ui/skeleton"
import { Table, TableBody, TableCell, TableRow } from "@lana/web/ui/table"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { Badge } from "@lana/web/ui/badge"

import { toast } from "sonner"

import { Button } from "@lana/web/ui/button"

import { useRouter } from "next/navigation"

import ChartOfAccountsUpload from "./upload"

import {
  useChartOfAccountsQuery,
  ChartNode,
  ChartOfAccountsQuery,
} from "@/lib/graphql/generated"

gql`
  query ChartOfAccounts {
    chartOfAccounts {
      id
      chartId
      name
      children {
        name
        accountCode
        children {
          name
          accountCode
          children {
            name
            accountCode
            children {
              name
              accountCode
              children {
                name
                accountCode
                children {
                  name
                  accountCode
                }
              }
            }
          }
        }
      }
    }
  }
`

const formatAccountCode = (code: string): string => {
  if (!code || typeof code !== "string") return ""
  const parts = code.split(".")
  return parts[parts.length - 1]
}

const LoadingSkeleton = () => {
  return (
    <Table data-testid="loading-skeleton">
      <TableBody>
        {[1, 2, 3].map((categoryIndex) => (
          <React.Fragment key={`category-${categoryIndex}`}>
            <TableRow>
              <TableCell className="text-primary">
                <Skeleton className="h-6 w-full" />
              </TableCell>
            </TableRow>
            {[1, 2, 3].map((accountIndex) => (
              <TableRow key={`account-${categoryIndex}-${accountIndex}`}>
                <TableCell className="pl-8">
                  <Skeleton className="h-5 w-full" />
                </TableCell>
              </TableRow>
            ))}
          </React.Fragment>
        ))}
      </TableBody>
    </Table>
  )
}

const getIndentLevel = (accountCode: string): number => {
  if (!accountCode.includes(".")) return 0
  return accountCode.split(".").length - 1
}

const getIndentClass = (accountCode: string): string => {
  const level = getIndentLevel(accountCode)
  switch (level) {
    case 0:
      return ""
    case 1:
      return "pl-6"
    case 2:
      return "pl-12"
    case 3:
      return "pl-18"
    case 4:
      return "pl-24"
    default:
      return `pl-[${Math.min(level * 8, 56)}]`
  }
}

const getTextClass = (accountCode: string): string => {
  const level = getIndentLevel(accountCode)
  if (level === 0) return "font-bold"
  if (level === 1) return ""
  return "text-sm"
}

const hasChildren = (account: ChartNode): boolean => {
  return Boolean(
    account &&
      account.children &&
      Array.isArray(account.children) &&
      account.children.length > 0,
  )
}

const hasDotChildren = (account: ChartNode): boolean => {
  if (!hasChildren(account)) return false
  return account.children!.some(
    (child) =>
      child &&
      child.accountCode &&
      typeof child.accountCode === "string" &&
      child.accountCode.includes("."),
  )
}

interface AccountRowProps {
  account: ChartNode
  hasDots: boolean
  isExpanded: boolean
  toggleExpand: () => void
}

const AccountRow = React.memo<AccountRowProps>(
  ({ account, hasDots, isExpanded, toggleExpand }) => {
    const t = useTranslations("ChartOfAccounts")
    const tCommon = useTranslations("Common")
    const [isHovering, setIsHovering] = useState(false)

    const router = useRouter()

    const onClick: MouseEventHandler<HTMLButtonElement> = (e) => {
      e.preventDefault()
      e.stopPropagation()
      router.push(`/chart-of-accounts/${account.accountCode}`)
    }

    return (
      <TableRow
        className={hasDots ? "cursor-pointer hover:bg-muted/5" : ""}
        onClick={hasDots ? toggleExpand : undefined}
        onMouseEnter={() => setIsHovering(true)}
        onMouseLeave={() => setIsHovering(false)}
      >
        <TableCell
          className={`${getIndentClass(account.accountCode)} flex justify-between`}
        >
          <div className="grid grid-cols-[100px_40px_1fr] items-center">
            <div>
              <Badge
                className="font-mono cursor-pointer"
                variant="secondary"
                onClick={(e) => {
                  e.stopPropagation()
                  const code = account.accountCode.replace(/\./g, "")
                  toast.info(t("copied", { code }))
                  navigator.clipboard.writeText(account.accountCode)
                }}
              >
                {formatAccountCode(account.accountCode)}
              </Badge>
            </div>
            <div className="flex justify-center">
              {hasDots ? (
                <span className="text-muted-foreground">
                  {isExpanded ? (
                    <IoCaretDownSharp className="h-4 w-4" />
                  ) : (
                    <IoCaretForwardSharp className="h-4 w-4" />
                  )}
                </span>
              ) : (
                <span className="w-4"></span>
              )}
            </div>
            <div className="flex space-x-4 items-center">
              <span className={getTextClass(account.accountCode)}>{account.name}</span>
              <div className="font-mono text-xs text-gray-500">
                ({account.accountCode})
              </div>
            </div>
          </div>
          {isHovering && (
            <Button
              variant="outline"
              size="sm"
              className="h-[22px] uppercase"
              onClick={onClick}
            >
              {tCommon("view")}
            </Button>
          )}
        </TableCell>
      </TableRow>
    )
  },
)
AccountRow.displayName = "AccountRow"

interface ChartOfAccountsViewProps {
  data?: ChartOfAccountsQuery | null
  loading: boolean
  error?: ApolloError
}

const ChartOfAccountsView: React.FC<ChartOfAccountsViewProps> = ({
  data,
  loading,
  error,
}) => {
  const [expandedAccounts, setExpandedAccounts] = useState<Record<string, boolean>>({})

  const toggleExpand = useCallback((accountCode: string) => {
    setExpandedAccounts((prev) => ({
      ...prev,
      [accountCode]: !prev[accountCode],
    }))
  }, [])

  if (loading && !data) return <LoadingSkeleton />
  if (error) return <p className="text-destructive">{error.message}</p>
  if (!data?.chartOfAccounts) return null

  const renderChartOfAccounts = () => {
    const result: React.ReactNode[] = []
    if (!data.chartOfAccounts.children || !Array.isArray(data.chartOfAccounts.children)) {
      return result
    }

    const queue = [...data.chartOfAccounts.children] as ChartNode[]
    const visited = new Set<string>()

    while (queue.length > 0) {
      const current = queue.shift()
      if (
        !current ||
        typeof current !== "object" ||
        !current.accountCode ||
        typeof current.accountCode !== "string"
      ) {
        continue
      }

      if (visited.has(current.accountCode)) continue
      visited.add(current.accountCode)
      const dotChildrenExist = hasDotChildren(current)
      const isExpanded = expandedAccounts[current.accountCode]

      result.push(
        <AccountRow
          key={current.accountCode}
          account={current}
          hasDots={dotChildrenExist}
          isExpanded={isExpanded}
          toggleExpand={() => toggleExpand(current.accountCode)}
        />,
      )

      if (hasChildren(current)) {
        const noDotChildren: ChartNode[] = []
        const dotChildren: ChartNode[] = []

        for (const child of current.children!) {
          if (!child || !child.accountCode || typeof child.accountCode !== "string")
            continue
          if (child.accountCode.includes(".")) {
            dotChildren.push(child)
          } else {
            noDotChildren.push(child)
          }
        }
        if (noDotChildren.length > 0) {
          queue.unshift(...noDotChildren)
        }
        if (isExpanded && dotChildren.length > 0) {
          queue.unshift(...dotChildren)
        }
      }
    }

    return result
  }

  return (
    <Table>
      <TableBody>{renderChartOfAccounts()}</TableBody>
    </Table>
  )
}

const ChartOfAccountsPage: React.FC = () => {
  const t = useTranslations("ChartOfAccounts")
  const {
    data: newChartData,
    loading: newChartLoading,
    error: newChartError,
  } = useChartOfAccountsQuery({
    fetchPolicy: "cache-and-network",
  })

  return (
    <Card className="mb-10">
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        {newChartData?.chartOfAccounts?.children &&
          newChartData.chartOfAccounts.children.length > 0 && (
            <ChartOfAccountsView
              data={newChartData}
              loading={newChartLoading}
              error={newChartError}
            />
          )}
        {newChartData?.chartOfAccounts?.chartId &&
          newChartData.chartOfAccounts.children.length === 0 && (
            <ChartOfAccountsUpload chartId={newChartData?.chartOfAccounts?.chartId} />
          )}
      </CardContent>
    </Card>
  )
}

export default ChartOfAccountsPage
