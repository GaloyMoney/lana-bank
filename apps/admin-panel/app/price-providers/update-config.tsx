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

import { gql } from "@apollo/client"

import {
  usePriceProviderConfigUpdateMutation,
  PriceProvidersDocument,
} from "@/lib/graphql/generated"

gql`
  mutation PriceProviderConfigUpdate($input: PriceProviderConfigUpdateInput!) {
    priceProviderConfigUpdate(input: $input) {
      priceProvider {
        id
        priceProviderId
        name
        provider
      }
    }
  }
`

interface UpdatePriceProviderConfigDialogProps {
  open: boolean
  setOpen: (open: boolean) => void
  priceProviderId: string
  provider: string
}

export const UpdatePriceProviderConfigDialog: React.FC<
  UpdatePriceProviderConfigDialogProps
> = ({ open, setOpen, priceProviderId, provider }) => {
  const t = useTranslations("PriceProviders.updateConfig")
  const tCommon = useTranslations("Common")

  const [name, setName] = useState("")
  const [usdPerBtc, setUsdPerBtc] = useState("")
  const [error, setError] = useState<string | null>(null)

  const resetForm = () => {
    setName("")
    setUsdPerBtc("")
    setError(null)
  }

  const closeDialog = () => {
    setOpen(false)
    resetForm()
  }

  const [updateConfig, { loading }] = usePriceProviderConfigUpdateMutation()

  const isBitfinex = provider === "bitfinex"
  const isManualPrice = provider === "manual-price"

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const config = isBitfinex
        ? { bitfinex: { name } }
        : { manualPrice: { usdCentsPerBtc: Math.round(Number(usdPerBtc) * 100) } }

      await updateConfig({
        variables: { input: { priceProviderId, config } },
        onCompleted: (data) => {
          if (data?.priceProviderConfigUpdate.priceProvider) {
            toast.success(t("success"))
            closeDialog()
          }
        },
        refetchQueries: [PriceProvidersDocument],
      })
    } catch (err) {
      console.error("Error updating price provider config:", err)
      if (err instanceof Error) {
        setError(err.message)
      } else {
        setError(tCommon("error"))
      }
    }
  }

  if (!isBitfinex && !isManualPrice) return null

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        setOpen(isOpen)
        if (!isOpen) resetForm()
      }}
    >
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          {isBitfinex && (
            <div>
              <Label htmlFor="price-provider-update-name">{t("fields.name")}</Label>
              <Input
                id="price-provider-update-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t("placeholders.name")}
                disabled={loading}
                required
                data-testid="price-provider-update-name-input"
              />
            </div>
          )}

          {isManualPrice && (
            <div>
              <Label htmlFor="price-provider-update-usd-per-btc">
                {t("fields.usdPerBtc")}
              </Label>
              <Input
                id="price-provider-update-usd-per-btc"
                type="number"
                value={usdPerBtc}
                onChange={(e) => setUsdPerBtc(e.target.value)}
                placeholder={t("placeholders.usdPerBtc")}
                disabled={loading}
                required
                min="0"
                step="0.01"
                data-testid="price-provider-update-usd-per-btc-input"
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
              data-testid="price-provider-update-config-cancel-button"
            >
              {tCommon("cancel")}
            </Button>
            <Button
              type="submit"
              loading={loading}
              data-testid="price-provider-update-config-submit-button"
            >
              {t("buttons.update")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
