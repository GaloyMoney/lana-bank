"use client"

import { useState } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"
import { Input } from "@lana/web/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"

import { gql } from "@apollo/client"

import {
  usePriceProviderCreateMutation,
  PriceProvidersDocument,
} from "@/lib/graphql/generated"

gql`
  mutation PriceProviderCreate($input: PriceProviderCreateInput!) {
    priceProviderCreate(input: $input) {
      priceProvider {
        id
        priceProviderId
        name
        createdAt
      }
    }
  }
`

type ProviderType = "bitfinex" | "manualPrice"

interface CreatePriceProviderDialogProps {
  openCreatePriceProviderDialog: boolean
  setOpenCreatePriceProviderDialog: (open: boolean) => void
}

export const CreatePriceProviderDialog: React.FC<CreatePriceProviderDialogProps> = ({
  openCreatePriceProviderDialog,
  setOpenCreatePriceProviderDialog,
}) => {
  const t = useTranslations("PriceProviders.create")
  const tCommon = useTranslations("Common")

  const [selectedType, setSelectedType] = useState<ProviderType>("bitfinex")
  const [name, setName] = useState("")
  const [usdPerBtc, setUsdPerBtc] = useState("")
  const [error, setError] = useState<string | null>(null)

  const resetForm = () => {
    setSelectedType("bitfinex")
    setName("")
    setUsdPerBtc("")
    setError(null)
  }

  const closeDialog = () => {
    setOpenCreatePriceProviderDialog(false)
    resetForm()
  }

  const [createPriceProvider, { loading }] = usePriceProviderCreateMutation()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const input =
        selectedType === "bitfinex"
          ? { bitfinex: { name } }
          : { manualPrice: { name, usdCentsPerBtc: Math.round(Number(usdPerBtc) * 100) } }

      await createPriceProvider({
        variables: { input },
        onCompleted: (data) => {
          if (data?.priceProviderCreate.priceProvider) {
            toast.success(t("success"))
            closeDialog()
          } else {
            throw new Error(t("errors.failed"))
          }
        },
        refetchQueries: [PriceProvidersDocument],
      })
    } catch (error) {
      console.error("Error creating price provider:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError(tCommon("error"))
      }
    }
  }

  return (
    <Dialog
      open={openCreatePriceProviderDialog}
      onOpenChange={(isOpen) => {
        setOpenCreatePriceProviderDialog(isOpen)
        if (!isOpen) resetForm()
      }}
    >
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <Label htmlFor="price-provider-type">{t("fields.type")}</Label>
            <Select
              value={selectedType}
              onValueChange={(value: ProviderType) => setSelectedType(value)}
              disabled={loading}
            >
              <SelectTrigger data-testid="price-provider-type-select">
                <SelectValue placeholder={t("placeholders.selectType")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="bitfinex">Bitfinex</SelectItem>
                <SelectItem value="manualPrice">Manual Price</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label htmlFor="price-provider-name">{t("fields.name")}</Label>
            <Input
              id="price-provider-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t("placeholders.name")}
              disabled={loading}
              required
              data-testid="price-provider-name-input"
            />
          </div>

          {selectedType === "manualPrice" && (
            <div>
              <Label htmlFor="price-provider-usd-per-btc">{t("fields.usdPerBtc")}</Label>
              <Input
                id="price-provider-usd-per-btc"
                type="number"
                value={usdPerBtc}
                onChange={(e) => setUsdPerBtc(e.target.value)}
                placeholder={t("placeholders.usdPerBtc")}
                disabled={loading}
                required
                min="0"
                step="0.01"
                data-testid="price-provider-usd-per-btc-input"
              />
            </div>
          )}

          {error && <div className="text-destructive text-sm">{error}</div>}
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={closeDialog}
              loading={loading}
              data-testid="price-provider-create-cancel-button"
            >
              {tCommon("cancel")}
            </Button>
            <Button
              type="submit"
              loading={loading}
              data-testid="price-provider-create-submit-button"
            >
              {t("buttons.create")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
