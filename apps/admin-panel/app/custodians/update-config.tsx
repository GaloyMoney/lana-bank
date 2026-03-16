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

import {
  type CustodianType,
  useCustodianConfigForm,
  ProviderConfigFields,
} from "./provider-config-fields"

import {
  useCustodianConfigUpdateMutation,
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

const mapProviderToType = (provider: string): CustodianType | null => {
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
  const tCommon = useTranslations("Common")

  const providerType = mapProviderToType(provider)
  const [error, setError] = useState<string | null>(null)
  const form = useCustodianConfigForm()

  const resetForm = () => {
    form.resetAll()
    setError(null)
  }

  const closeDialog = () => {
    setOpen(false)
    resetForm()
  }

  const [updateConfig, { loading }] = useCustodianConfigUpdateMutation()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!providerType) return

    try {
      const config = form.buildConfigInput(providerType)
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
          <ProviderConfigFields
            type={providerType}
            form={form}
            loading={loading}
            testIdPrefix="custodian-update"
          />

          {error && <div className="text-destructive text-sm">{error}</div>}
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={closeDialog}
              loading={loading}
              data-testid="custodian-update-config-cancel-button"
            >
              {tCommon("cancel")}
            </Button>
            <Button
              type="submit"
              loading={loading}
              data-testid="custodian-update-config-submit-button"
            >
              {t("buttons.update")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
