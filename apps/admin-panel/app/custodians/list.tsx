"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"
import { Button } from "@lana/web/ui/button"

import { UpdateCustodianConfigDialog } from "./update-config"

import {
  Custodian,
  CustodiansSort,
  SortDirection,
  useCustodiansQuery,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { camelToScreamingSnake } from "@/lib/utils"

gql`
  fragment CustodianFields on Custodian {
    id
    custodianId
    createdAt
    name
    provider
  }

  query Custodians($first: Int!, $after: String, $sort: CustodiansSort) {
    custodians(first: $first, after: $after, sort: $sort) {
      edges {
        cursor
        node {
          ...CustodianFields
        }
      }
      pageInfo {
        endCursor
        startCursor
        hasNextPage
        hasPreviousPage
      }
    }
  }
`

const CustodiansList = () => {
  const t = useTranslations("Custodians.table")
  const tUpdate = useTranslations("Custodians.updateConfig")
  const [sortBy, setSortBy] = useState<CustodiansSort | null>(null)
  const [selectedCustodian, setSelectedCustodian] = useState<{
    id: string
    provider: string
  } | null>(null)
  const [openUpdateDialog, setOpenUpdateDialog] = useState(false)

  const { data, loading, error, fetchMore } = useCustodiansQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
    },
  })

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<Custodian>
        columns={columns(t, tUpdate, (custodian) => {
          setSelectedCustodian({
            id: custodian.custodianId,
            provider: custodian.provider,
          })
          setOpenUpdateDialog(true)
        })}
        data={data?.custodians as PaginatedData<Custodian>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as CustodiansSort["by"],
            direction: direction as SortDirection,
          })
        }}
      />
      {selectedCustodian && (
        <UpdateCustodianConfigDialog
          open={openUpdateDialog}
          setOpen={setOpenUpdateDialog}
          custodianId={selectedCustodian.id}
          provider={selectedCustodian.provider}
        />
      )}
    </div>
  )
}

export default CustodiansList

const columns = (
  t: ReturnType<typeof useTranslations>,
  tUpdate: ReturnType<typeof useTranslations>,
  onUpdateConfig: (custodian: Custodian) => void,
): Column<Custodian>[] => [
  {
    key: "name",
    label: t("headers.name"),
    sortable: true,
  },
  {
    key: "provider",
    label: t("headers.provider"),
  },
  {
    key: "createdAt",
    label: t("headers.created"),
    render: (createdAt) => <DateWithTooltip value={createdAt} />,
    sortable: true,
  },
  {
    key: "isManual",
    label: "",
    render: (isManual, record) =>
      !isManual ? (
        <Button
          variant="outline"
          size="sm"
          onClick={(e) => {
            e.stopPropagation()
            onUpdateConfig(record)
          }}
        >
          {tUpdate("buttons.update")}
        </Button>
      ) : null,
  },
]
