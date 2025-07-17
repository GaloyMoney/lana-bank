"use client"

import { useRouter } from "next/navigation"
import { useTranslations } from "next-intl"

import { CommandGroup, CommandItem, CommandSeparator } from "@lana/web/ui/command"

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
  primary: string
  secondary: string
  id: string
}

const getResultInfo = (result: SearchResult): ResultInfo | null => {
  switch (result.__typename) {
    case "DepositAccount":
      return {
        url: `/customers/${result.customer.publicId}`,
        primary: result.customer.email,
        secondary: `Deposit Account`,
        id: result.id,
      }
    case "Customer":
      return {
        url: `/customers/${result.publicId}`,
        primary: result.email,
        secondary: `Customer`,
        id: result.id,
      }
    default:
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
    const info = getResultInfo(result)
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
  const info = getResultInfo(result)

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
