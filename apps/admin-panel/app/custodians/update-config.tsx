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

import { gql } from "@apollo/client"

import { useCustodianFormState } from "./shared/use-custodian-form-state"
import {
  KomainuFormFields,
  BitgoFormFields,
  SelfCustodyFormFields,
} from "./shared/custodian-form-fields"

import {
  useCustodianConfigUpdateMutation,
  type CustodianConfigInput,
  CustodiansDocument,
} from "@/lib/graphql/generated"

gql`
  mutation CustodianConfigUpdate($input: CustodianConfigUpdateInput!) {
    custodianConfigUpdate(input: $input) {
      custodian {
        id
        custodianId
        name
        provider
      }
    }
  }
`

interface UpdateCustodianConfigDialogProps {
  open: boolean
  setOpen: (open: boolean) => void
  custodianId: string
  provider: string
}

type ProviderType = "komainu" | "bitgo" | "selfCustody"

const mapProviderToType = (provider: string): ProviderType | null => {
  const lower = provider.toLowerCase()
  if (lower === "komainu") return "komainu"
  if (lower === "bitgo") return "bitgo"
  if (lower === "self-custody" || lower === "selfcustody" || lower === "self_custody")
    return "selfCustody"
  return null
}

export const UpdateCustodianConfigDialog: React.FC<UpdateCustodianConfigDialogProps> = ({
  open,
  setOpen,
  custodianId,
  provider,
}) => {
  const t = useTranslations("Custodians.updateConfig")
  const tFields = useTranslations("Custodians.create.fields")
  const tPlaceholders = useTranslations("Custodians.create.placeholders")
  const tCommon = useTranslations("Common")

  const providerType = mapProviderToType(provider)

  const {
    komainuConfig,
    bitgoConfig,
    selfCustodyConfig,
    handleKomainuInputChange,
    handleBitgoInputChange,
    handleSelfCustodyInputChange,
    handleKomainuCheckboxChange,
    handleBitgoCheckboxChange,
    handleSelfCustodyNetworkChange,
    resetProviderConfigs,
  } = useCustodianFormState()

  const [error, setError] = useState<string | null>(null)

  const resetForm = () => {
    resetProviderConfigs()
    setError(null)
  }

  const closeDialog = () => {
    setOpen(false)
    resetForm()
  }

  const [updateConfig, { loading }] = useCustodianConfigUpdateMutation()

  const buildConfigInput = (): CustodianConfigInput | null => {
    switch (providerType) {
      case "komainu":
        return { komainu: komainuConfig }
      case "bitgo":
        return { bitgo: bitgoConfig }
      case "selfCustody":
        return { selfCustody: selfCustodyConfig }
      default:
        return null
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    const config = buildConfigInput()
    if (!config) return

    try {
      await updateConfig({
        variables: { input: { custodianId, config } },
        onCompleted: (data) => {
          if (data?.custodianConfigUpdate.custodian) {
            toast.success(t("success"))
            closeDialog()
          }
        },
        refetchQueries: [CustodiansDocument],
      })
    } catch (err) {
      console.error("Error updating custodian config:", err)
      if (err instanceof Error) {
        setError(err.message)
      } else {
        setError(tCommon("error"))
      }
    }
  }

  if (!providerType) return null

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
          {providerType === "komainu" && (
            <KomainuFormFields
              config={komainuConfig}
              onInputChange={handleKomainuInputChange}
              onCheckboxChange={handleKomainuCheckboxChange}
              loading={loading}
              tFields={tFields}
              tPlaceholders={tPlaceholders}
            />
          )}

          {providerType === "bitgo" && (
            <BitgoFormFields
              config={bitgoConfig}
              onInputChange={handleBitgoInputChange}
              onCheckboxChange={handleBitgoCheckboxChange}
              loading={loading}
              tFields={tFields}
              tPlaceholders={tPlaceholders}
            />
          )}

          {providerType === "selfCustody" && (
            <SelfCustodyFormFields
              config={selfCustodyConfig}
              onInputChange={handleSelfCustodyInputChange}
              onNetworkChange={handleSelfCustodyNetworkChange}
              loading={loading}
              tFields={tFields}
              tPlaceholders={tPlaceholders}
            />
          )}

          {error && <div className="text-destructive text-sm">{error}</div>}
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={closeDialog}
              loading={loading}
            >
              {tCommon("cancel")}
            </Button>
            <Button type="submit" loading={loading}>
              {t("buttons.update")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
