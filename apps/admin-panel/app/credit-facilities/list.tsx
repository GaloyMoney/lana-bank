"use client"

import { gql } from "@apollo/client"
import { useState } from "react"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { LoanAndCreditFacilityStatusBadge } from "./status-badge"

import { CollateralizationStateLabel } from "./label"

import {
  CreditFacilitiesSort,
  CreditFacility,
  SortDirection,
  CreditFacilityStatus,
  CollateralizationState,
  CreditFacilitiesFilter,
  useCreditFacilitiesQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import Balance from "@/components/balance/balance"

import { camelToScreamingSnake, formatDate } from "@/lib/utils"

import { UsdCents } from "@/types"

import DateWithTooltip from "@/components/date-with-tooltip"

gql`
  query CreditFacilities(
    $first: Int!
    $after: String
    $sort: CreditFacilitiesSort
    $filter: CreditFacilitiesFilter
  ) {
    creditFacilities(first: $first, after: $after, sort: $sort, filter: $filter) {
      edges {
        cursor
        node {
          id
          creditFacilityId
          collateralizationState
          createdAt
          status
          facilityAmount
          creditFacilityTerms {
            annualRate
            accrualInterval
            oneTimeFeeRate
          }
          currentCvl
          balance {
            collateral {
              btcBalance
            }
            outstanding {
              usdBalance
            }
          }
          repaymentPlan {
            repaymentType
            status
            initial
            outstanding
            dueAt
          }
          customer {
            customerId
            email
          }
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }
`

const CreditFacilities = () => {
  const t = useTranslations("CreditFacilities")
  const [sortBy, setSortBy] = useState<CreditFacilitiesSort | null>(null)
  const [filter, setFilter] = useState<CreditFacilitiesFilter | null>(null)

  const { data, loading, error, fetchMore } = useCreditFacilitiesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
      filter: filter,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{t("errors.general")}</p>}
      <PaginatedTable<CreditFacility>
        columns={columns(t)}
        data={data?.creditFacilities as PaginatedData<CreditFacility>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(facility) => `/credit-facilities/${facility.creditFacilityId}`}
        onSort={(column, direction) => {
          setSortBy({
            by: (column === "currentCvl"
              ? "CVL"
              : camelToScreamingSnake(column)) as CreditFacilitiesSort["by"],
            direction: direction as SortDirection,
          })
        }}
        onFilter={(column, value) => {
          if (value)
            setFilter({
              field: camelToScreamingSnake(column) as CreditFacilitiesFilter["field"],
              [column]: value,
            })
          else setFilter(null)
        }}
      />
    </div>
  )
}

export default CreditFacilities

const columns = (t: (key: string) => string): Column<CreditFacility>[] => [
  {
    key: "customer",
    label: t("table.headers.customer"),
    render: (customer) => customer.email,
  },
  {
    key: "status",
    label: t("table.headers.status"),
    render: (status) => (
      <LoanAndCreditFacilityStatusBadge
        className="flex items-center justify-center text-center min-h-full min-w-full"
        status={status}
      />
    ),
    filterValues: Object.values(CreditFacilityStatus),
  },
  {
    key: "status",
    label: t("table.headers.state"),
    render: (status) => {
      switch (status) {
        case CreditFacilityStatus.Active:
          return "active"
        case CreditFacilityStatus.Matured:
          return "matured"
        case CreditFacilityStatus.Closed:
          return "closed"
        case CreditFacilityStatus.PendingApproval:
          return "pending approval"
        case CreditFacilityStatus.PendingCollateralization:
          return "pending collateral"
        default:
          return "-"
      }
    },
  },
  {
    key: "balance",
    label: t("table.headers.outstanding"),
    render: (balance) => (
      <Balance amount={balance.outstanding.usdBalance} currency="usd" />
    ),
  },
  {
    key: "repaymentPlan",
    label: t("table.headers.monthlyPayment"),
    render: (_, facility) => {
      const monthlyPayment = (facility.repaymentPlan
        ?.filter(
          (payment) => payment.status === "UPCOMING" || payment.status === "NOT_YET_DUE",
        )
        .reduce((acc, payment) => acc + payment.initial, 0) / 12) as UsdCents

      return <Balance amount={monthlyPayment || (0 as UsdCents)} currency="usd" />
    },
  },
  {
    key: "collateralizationState",
    label: t("table.headers.collateralizationState"),
    render: (state) => <CollateralizationStateLabel state={state} />,
    filterValues: Object.values(CollateralizationState),
  },
  {
    key: "creditFacilityTerms",
    label: t("table.headers.nominalRate"),
    render: (terms) => {
      if (!terms) return "-"
      return `${terms.annualRate}% ${terms.accrualInterval.toLowerCase()}`
    },
  },
  {
    key: "creditFacilityTerms",
    label: t("table.headers.effectiveRate"),
    render: (terms) => {
      if (!terms) return "-"
      const effectiveRate = terms.annualRate + (terms.oneTimeFeeRate || 0)
      return `${effectiveRate.toFixed(2)}%`
    },
  },
  {
    key: "currentCvl",
    label: t("table.headers.cvl"),
    render: (cvl) => `${cvl}%`,
    sortable: true,
  },
  {
    key: "createdAt",
    label: t("table.headers.createdAt"),
    render: (date) => <DateWithTooltip value={date} />,
    sortable: true,
  },
  {
    key: "createdAt",
    label: t("table.headers.interestDays"),
    render: () => {
      const year = new Date().getFullYear()
      return year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0) ? "366" : "365"
    },
  },
]
