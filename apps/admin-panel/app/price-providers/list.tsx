"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"
import { Button } from "@lana/web/ui/button"
import { Badge } from "@lana/web/ui/badge"

import { UpdatePriceProviderConfigDialog } from "./update-config"

import {
  PriceProvider,
  PriceProvidersSort,
  SortDirection,
  PriceProvidersDocument,
  usePriceProvidersQuery,
  usePriceProviderActivateMutation,
  usePriceProviderDeactivateMutation,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { camelToScreamingSnake } from "@/lib/utils"

gql`
  fragment PriceProviderFields on PriceProvider {
    id
    priceProviderId
    createdAt
    name
    provider
    active
  }

  query PriceProviders($first: Int!, $after: String, $sort: PriceProvidersSort) {
    priceProviders(first: $first, after: $after, sort: $sort) {
      edges {
        cursor
        node {
          ...PriceProviderFields
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

  mutation PriceProviderActivate($priceProviderId: UUID!) {
    priceProviderActivate(priceProviderId: $priceProviderId) {
      priceProvider {
        id
        priceProviderId
        active
      }
    }
  }

  mutation PriceProviderDeactivate($priceProviderId: UUID!) {
    priceProviderDeactivate(priceProviderId: $priceProviderId) {
      priceProvider {
        id
        priceProviderId
        active
      }
    }
  }
`

const PriceProvidersList = () => {
  const t = useTranslations("PriceProviders.table")
  const [sortBy, setSortBy] = useState<PriceProvidersSort | null>(null)
  const [selectedProvider, setSelectedProvider] = useState<{
    id: string
    provider: string
  } | null>(null)
  const [openUpdateDialog, setOpenUpdateDialog] = useState(false)

  const { data, loading, error, fetchMore } = usePriceProvidersQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
    },
  })

  const [activate] = usePriceProviderActivateMutation()
  const [deactivate] = usePriceProviderDeactivateMutation()

  const handleActivate = async (priceProviderId: string) => {
    try {
      await activate({
        variables: { priceProviderId },
        refetchQueries: [PriceProvidersDocument],
      })
      toast.success(t("success.activated"))
    } catch {
      toast.error(t("errors.activateFailed"))
    }
  }

  const handleDeactivate = async (priceProviderId: string) => {
    try {
      await deactivate({
        variables: { priceProviderId },
        refetchQueries: [PriceProvidersDocument],
      })
      toast.success(t("success.deactivated"))
    } catch {
      toast.error(t("errors.deactivateFailed"))
    }
  }

  return (
    <div>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<PriceProvider>
        columns={columns(t, handleActivate, handleDeactivate, (provider) => {
          setSelectedProvider({
            id: provider.priceProviderId,
            provider: provider.provider,
          })
          setOpenUpdateDialog(true)
        })}
        data={data?.priceProviders as PaginatedData<PriceProvider>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        onSort={(column, direction) => {
          setSortBy({
            by: camelToScreamingSnake(column) as PriceProvidersSort["by"],
            direction: direction as SortDirection,
          })
        }}
      />
      {selectedProvider && (
        <UpdatePriceProviderConfigDialog
          open={openUpdateDialog}
          setOpen={setOpenUpdateDialog}
          priceProviderId={selectedProvider.id}
          provider={selectedProvider.provider}
        />
      )}
    </div>
  )
}

export default PriceProvidersList

const columns = (
  t: ReturnType<typeof useTranslations>,
  onActivate: (id: string) => void,
  onDeactivate: (id: string) => void,
  onUpdateConfig: (provider: PriceProvider) => void,
): Column<PriceProvider>[] => [
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
    key: "active",
    label: t("headers.status"),
    render: (active) => (
      <Badge variant={active ? "success" : "secondary"}>
        {active ? t("status.active") : t("status.inactive")}
      </Badge>
    ),
  },
  {
    key: "createdAt",
    label: t("headers.created"),
    render: (createdAt) => <DateWithTooltip value={createdAt} />,
    sortable: true,
  },
  {
    key: "id",
    label: "",
    render: (_id, record) => (
      <div className="flex gap-2">
        {record.active ? (
          <Button
            variant="outline"
            size="sm"
            onClick={(e) => {
              e.stopPropagation()
              onDeactivate(record.priceProviderId)
            }}
            data-testid="price-provider-deactivate-button"
          >
            {t("actions.deactivate")}
          </Button>
        ) : (
          <Button
            variant="outline"
            size="sm"
            onClick={(e) => {
              e.stopPropagation()
              onActivate(record.priceProviderId)
            }}
            data-testid="price-provider-activate-button"
          >
            {t("actions.activate")}
          </Button>
        )}
        <Button
          variant="outline"
          size="sm"
          onClick={(e) => {
            e.stopPropagation()
            onUpdateConfig(record)
          }}
          data-testid="price-provider-update-config-button"
        >
          {t("actions.updateConfig")}
        </Button>
      </div>
    ),
  },
]
