"use client"

import React, { useState, useCallback, MouseEventHandler } from "react"
import { ApolloError, gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { IoAddSharp, IoCaretDownSharp, IoCaretForwardSharp } from "react-icons/io5"

import { Skeleton } from "@lana/web/ui/skeleton"
import { Table, TableBody, TableCell, TableRow } from "@lana/web/ui/table"
import { Button } from "@lana/web/ui/button"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { Badge } from "@lana/web/ui/badge"

import { toast } from "sonner"

import { useRouter } from "next/navigation"

import ChartOfAccountsUpload from "./upload"
import { AddChartNodeDialog } from "./add-node"

import {
  useChartOfAccountsQuery,
  ChartNode,
  ChartOfAccountsQuery,
} from "@/lib/graphql/generated"

gql`
  fragment ChartAccountBase on ChartNode {
    name
    accountCode
  }

  fragment ChartOfAccountsFields on ChartOfAccounts {
    id
    chartId
    name
    children {
      ...ChartAccountBase
      children {
        ...ChartAccountBase
        children {
          ...ChartAccountBase
          children {
            ...ChartAccountBase
            children {
              ...ChartAccountBase
              children {
                ...ChartAccountBase
              }
            }
          }
        }
      }
    }
  }

  query ChartOfAccounts {
    chartOfAccounts {
      ...ChartOfAccountsFields
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
      return "pl-24"
    case 4:
      return "pl-32"
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
  onAddChild: (parentCode: string) => void
}

const AccountRow = React.memo<AccountRowProps>(
  ({ account, hasDots, isExpanded, toggleExpand, onAddChild }) => {
    const t = useTranslations("ChartOfAccounts")
    const [isHovered, setIsHovered] = useState(false)
    const router = useRouter()

    const onClick: MouseEventHandler<HTMLTableRowElement> = (e) => {
      e.preventDefault()
      e.stopPropagation()
      router.push(`/ledger-accounts/${account.accountCode}`)
    }

    const handleAddChild = (e: React.MouseEvent) => {
      e.preventDefault()
      e.stopPropagation()
      onAddChild(account.accountCode)
    }

    return (
      <TableRow
        className="cursor-pointer group"
        onClick={onClick}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
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
                <span
                  onClick={(e) => {
                    e.stopPropagation()
                    if (hasDots) toggleExpand()
                  }}
                  className="text-muted-foreground cursor-pointer hover:bg-muted p-1 rounded-full"
                >
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
            <span className={getTextClass(account.accountCode)}>{account.name}</span>
          </div>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="ghost"
              className={`h-6 w-6 p-0 transition-opacity ${
                isHovered ? "opacity-100" : "opacity-0"
              }`}
              onClick={handleAddChild}
              data-testid={`add-child-${account.accountCode}`}
            >
              <IoAddSharp className="h-3 w-3" />
            </Button>
            <div className="font-mono text-xs text-gray-500">{account.accountCode}</div>
          </div>
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
  onAddChild: (parentCode: string) => void
}

const ChartOfAccountsView: React.FC<ChartOfAccountsViewProps> = ({
  data,
  loading,
  error,
  onAddChild,
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
          onAddChild={onAddChild}
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
  const [openAddNodeDialog, setOpenAddNodeDialog] = useState(false)
  const [parentCodeForNewNode, setParentCodeForNewNode] = useState<string | undefined>()

  const {
    data: newChartData,
    loading: newChartLoading,
    error: newChartError,
  } = useChartOfAccountsQuery({
    fetchPolicy: "cache-and-network",
  })

  const chartId = newChartData?.chartOfAccounts?.chartId

  const handleAddChild = (parentCode: string) => {
    setParentCodeForNewNode(parentCode)
    setOpenAddNodeDialog(true)
  }

  const handleOpenAddNode = () => {
    setParentCodeForNewNode(undefined)
    setOpenAddNodeDialog(true)
  }

  return (
    <>
      <Card className="mb-10">
        <CardHeader>
          <div className="flex justify-between items-start">
            <div className="flex flex-col gap-1.5">
              <CardTitle>{t("title")}</CardTitle>
              <CardDescription>{t("description")}</CardDescription>
            </div>
            {chartId && (
              <Button
                variant="outline"
                onClick={handleOpenAddNode}
                data-testid="add-chart-node-button"
              >
                {t("addNode")}
              </Button>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {chartId && (
            <>
              {newChartData.chartOfAccounts.children.length > 0 ? (
                <ChartOfAccountsView
                  data={newChartData}
                  loading={newChartLoading}
                  error={newChartError}
                  onAddChild={handleAddChild}
                />
              ) : (
                <ChartOfAccountsUpload chartId={chartId} />
              )}
            </>
          )}
        </CardContent>
      </Card>

      {chartId && (
        <AddChartNodeDialog
          openAddNodeDialog={openAddNodeDialog}
          setOpenAddNodeDialog={setOpenAddNodeDialog}
          chartId={chartId}
          parentCode={parentCodeForNewNode}
        />
      )}
    </>
  )
}

export default ChartOfAccountsPage
