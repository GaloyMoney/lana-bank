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
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"

import { gql } from "@apollo/client"

import { useCustodianFormState } from "./shared/use-custodian-form-state"
import {
  KomainuFormFields,
  BitgoFormFields,
  SelfCustodyFormFields,
} from "./shared/custodian-form-fields"

import { useManualCustodianEnabled } from "@/hooks/use-manual-custodian-enabled"
import {
  useCustodianCreateMutation,
  type ManualConfig,
  type CustodianCreateInput,
  CustodiansDocument,
} from "@/lib/graphql/generated"

gql`
  mutation CustodianCreate($input: CustodianCreateInput!) {
    custodianCreate(input: $input) {
      custodian {
        id
        custodianId
        name
        createdAt
      }
    }
  }
`

type CustodianType = "komainu" | "bitgo" | "selfCustody" | "manual"

interface CreateCustodianDialogProps {
  openCreateCustodianDialog: boolean
  setOpenCreateCustodianDialog: (open: boolean) => void
}

export const CreateCustodianDialog: React.FC<CreateCustodianDialogProps> = ({
  openCreateCustodianDialog,
  setOpenCreateCustodianDialog,
}) => {
  const t = useTranslations("Custodians.create")
  const tCommon = useTranslations("Common")

  const manualCustodianEnabled = useManualCustodianEnabled()

  const [selectedType, setSelectedType] = useState<CustodianType>("komainu")

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

  const [manualConfig, setManualConfig] = useState<ManualConfig>({
    name: "",
  })
  const [error, setError] = useState<string | null>(null)

  const resetForm = () => {
    setSelectedType("komainu")
    resetProviderConfigs()
    setManualConfig({
      name: "",
    })
    setError(null)
  }

  const closeDialog = () => {
    setOpenCreateCustodianDialog(false)
    resetForm()
  }

  const [createCustodian, { loading }] = useCustodianCreateMutation()

  const handleManualInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target
    setManualConfig((prev) => ({ ...prev, [name]: value }))
  }

  const buildCustodianInput = (): CustodianCreateInput => {
    switch (selectedType) {
      case "komainu":
        return { komainu: komainuConfig }
      case "bitgo":
        return { bitgo: bitgoConfig }
      case "selfCustody":
        return { selfCustody: selfCustodyConfig }
      case "manual":
        return { manual: manualConfig }
      default:
        throw new Error(`Unsupported custodian type: ${selectedType}`)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const input = buildCustodianInput()
      await createCustodian({
        variables: { input },
        onCompleted: (data) => {
          if (data?.custodianCreate.custodian) {
            toast.success(t("success"))
            closeDialog()
          } else {
            throw new Error(t("errors.failed"))
          }
        },
        refetchQueries: [CustodiansDocument],
      })
    } catch (error) {
      console.error("Error creating custodian:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError(tCommon("error"))
      }
    }
  }

  return (
    <Dialog
      open={openCreateCustodianDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateCustodianDialog(isOpen)
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
            <Label htmlFor="custodian-type">{t("fields.type")}</Label>
            <Select
              value={selectedType}
              onValueChange={(value: CustodianType) => setSelectedType(value)}
              disabled={loading}
            >
              <SelectTrigger data-testid="custodian-type-select">
                <SelectValue placeholder={t("placeholders.selectType")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="komainu">Komainu</SelectItem>
                <SelectItem value="bitgo">BitGo</SelectItem>
                <SelectItem value="selfCustody">{t("fields.selfCustodyLabel")}</SelectItem>
                {manualCustodianEnabled && (
                  <SelectItem value="manual">{t("fields.manualLabel")}</SelectItem>
                )}
              </SelectContent>
            </Select>
          </div>

          {selectedType === "komainu" && (
            <KomainuFormFields
              config={komainuConfig}
              onInputChange={handleKomainuInputChange}
              onCheckboxChange={handleKomainuCheckboxChange}
              loading={loading}
              tFields={(key) => t(`fields.${key}`)}
              tPlaceholders={(key) => t(`placeholders.${key}`)}
              dataTestId={true}
            />
          )}

          {selectedType === "bitgo" && (
            <BitgoFormFields
              config={bitgoConfig}
              onInputChange={handleBitgoInputChange}
              onCheckboxChange={handleBitgoCheckboxChange}
              loading={loading}
              tFields={(key) => t(`fields.${key}`)}
              tPlaceholders={(key) => t(`placeholders.${key}`)}
              dataTestId={true}
            />
          )}

          {selectedType === "selfCustody" && (
            <SelfCustodyFormFields
              config={selfCustodyConfig}
              onInputChange={handleSelfCustodyInputChange}
              onNetworkChange={handleSelfCustodyNetworkChange}
              loading={loading}
              tFields={(key) => t(`fields.${key}`)}
              tPlaceholders={(key) => t(`placeholders.${key}`)}
              dataTestId={true}
            />
          )}
          {selectedType === "manual" && (
            <>
              <div>
                <Label htmlFor="name" required>
                  {t("fields.name")}
                </Label>
                <Input
                  id="name"
                  name="name"
                  value={manualConfig.name}
                  onChange={handleManualInputChange}
                  placeholder={t("placeholders.name")}
                  required
                  disabled={loading}
                  data-testid="custodian-name-input"
                />
              </div>
            </>
          )}
          {error && <div className="text-destructive text-sm">{error}</div>}
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={closeDialog}
              loading={loading}
              data-testid="custodian-create-cancel-button"
            >
              {tCommon("cancel")}
            </Button>
            <Button
              type="submit"
              loading={loading}
              data-testid="custodian-create-submit-button"
            >
              {t("buttons.create")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
