import { useState } from "react"

import { gql } from "@apollo/client"

import {
  SearchPublicIdTargetQuery,
  useSearchPublicIdTargetLazyQuery,
} from "@/lib/graphql/generated"

gql`
  query SearchPublicIdTarget($publicId: PublicId!) {
    publicIdTarget(id: $publicId) {
      __typename
      ... on Customer {
        id
        customerId
        publicId
        email
      }
      ... on Prospect {
        id
        prospectId
        publicId
        email
      }
      ... on DepositAccount {
        id
        publicId
      }
      ... on CreditFacility {
        id
        publicId
        facilityAmount
      }
      ... on CreditFacilityDisbursal {
        id
        amount
        publicId
      }
      ... on Deposit {
        id
        amount
        publicId
        depositId
      }
      ... on Withdrawal {
        id
        amount
        publicId
        withdrawalId
      }
    }
  }
`

type SearchResult = NonNullable<SearchPublicIdTargetQuery["publicIdTarget"]>

export function usePublicIdSearch() {
  const [searchTerm, setSearchTerm] = useState("")
  const [searchResults, setSearchResults] = useState<SearchResult | null>(null)
  const [isSearching, setIsSearching] = useState(false)

  const [searchPublicIdTarget, { loading: searchLoading }] =
    useSearchPublicIdTargetLazyQuery({
      onCompleted: (data) => {
        setIsSearching(false)
        if (data.publicIdTarget) {
          setSearchResults(data.publicIdTarget)
        } else {
          setSearchResults(null)
        }
      },
      onError: () => {
        setIsSearching(false)
        setSearchResults(null)
      },
    })

  const handleSearchInputChange = (value: string) => {
    setSearchTerm(value)
    setSearchResults(null)
    setIsSearching(false)
    if (value.trim() && /^\d/.test(value.trim())) {
      setIsSearching(true)
      searchPublicIdTarget({
        variables: {
          publicId: value.trim(),
        },
      })
    }
  }

  const clearSearch = () => {
    setSearchTerm("")
    setSearchResults(null)
    setIsSearching(false)
  }

  const isSearchMode = Boolean(searchTerm.trim() && /^\d/.test(searchTerm.trim()))
  const hasResults = searchResults !== null
  const showNoResults = Boolean(
    isSearchMode && !hasResults && !isSearching && !searchLoading,
  )

  return {
    searchTerm,
    searchResults,
    isSearching: isSearching || searchLoading,
    isSearchMode,
    hasResults,
    showNoResults,

    handleSearchInputChange,
    clearSearch,
  }
}
