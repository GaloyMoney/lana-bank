"use client"

import { useRouter } from "next/navigation"
import { useTranslations } from "next-intl"

import { CommandGroup, CommandItem, CommandSeparator } from "@lana/web/ui/command"

import Balance from "../balance/balance"

import { SearchPublicIdTargetQuery } from "@/lib/graphql/generated"

type SearchResult = NonNullable<SearchPublicIdTargetQuery["publicIdTarget"]>

interface SearchResultsProps {
  results: SearchResult
  isSearching: boolean
  showNoResults: boolean
  onResultSelect: () => void
}

type ResultInfo = {
  url: string
  primary: React.ReactNode
  secondary: string
  id: string
}

const getResultInfo = (
  result: SearchResult,
  t: ReturnType<typeof useTranslations<"CommandMenu">>,
): ResultInfo | null => {
  switch (result.__typename) {
    case "DepositAccount":
      return {
        url: `/deposit-accounts/${result.publicId}`,
        primary: result.customerId,
        secondary: t("searchResultTypes.depositAccount"),
        id: result.publicId,
      }
    case "Customer":
      return {
        url: `/customers/${result.publicId}`,
        primary: result.email,
        secondary: t("searchResultTypes.customer"),
        id: result.publicId,
      }
    case "CreditFacility":
      return {
        url: `/credit-facilities/${result.publicId}`,
        primary: (
          <div className="flex items-center gap-2">
            <span>{t("searchResultTypes.facilityAmount")}</span>
            <Balance amount={result.facilityAmount} currency="usd" />
          </div>
        ),
        secondary: t("searchResultTypes.creditFacility"),
        id: result.publicId,
      }
    case "CreditFacilityDisbursal":
      return {
        url: `/disbursals/${result.publicId}`,
        primary: (
          <div className="flex items-center gap-2">
            <span>{t("searchResultTypes.amount")}</span>
            <Balance amount={result.amount} currency="usd" />
          </div>
        ),
        secondary: t("searchResultTypes.disbursal"),
        id: result.publicId,
      }
    case "Deposit":
      return {
        url: `/deposits/${result.publicId}`,
        primary: (
          <div className="flex items-center gap-2">
            <span>{t("searchResultTypes.amount")}</span>
            <Balance amount={result.amount} currency="usd" />
          </div>
        ),
        secondary: t("searchResultTypes.deposit"),
        id: result.publicId,
      }
    case "Withdrawal":
      return {
        url: `/withdrawals/${result.publicId}`,
        primary: (
          <div className="flex items-center gap-2">
            <span>{t("searchResultTypes.amount")}</span>
            <Balance amount={result.amount} currency="usd" />
          </div>
        ),
        secondary: t("searchResultTypes.withdrawal"),
        id: result.publicId,
      }
    case "Prospect":
      return {
        url: `/prospects/${result.publicId}`,
        primary: result.email,
        secondary: t("searchResultTypes.prospect"),
        id: result.publicId,
      }
    default:
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const _exhaustiveCheck: never = result
      return null
  }
}

export function SearchResults({
  results,
  isSearching,
  showNoResults,
  onResultSelect,
}: SearchResultsProps) {
  const router = useRouter()
  const t = useTranslations("CommandMenu")

  const handleResultClick = (result: SearchResult) => {
    const info = getResultInfo(result, t)
    if (!info) {
      return
    }
    router.push(info.url)
    onResultSelect()
  }

  if (!results && !isSearching && !showNoResults) {
    return null
  }

  return (
    <>
      <CommandSeparator />
      <CommandGroup heading={t("headings.searchResults")}>
        {results && (
          <SearchResultItem
            result={results}
            onSelect={() => handleResultClick(results)}
          />
        )}
      </CommandGroup>
    </>
  )
}

interface SearchResultItemProps {
  result: SearchResult
  onSelect: () => void
}

function SearchResultItem({ result, onSelect }: SearchResultItemProps) {
  const t = useTranslations("CommandMenu")
  const info = getResultInfo(result, t)

  if (!info) {
    return null
  }

  return (
    <CommandItem onSelect={onSelect} className="flex items-center gap-2" value={info.id}>
      <div className="flex flex-col">
        <span className="font-medium">{info.primary}</span>
        <span className="text-sm text-muted-foreground">{info.secondary}</span>
      </div>
    </CommandItem>
  )
}
