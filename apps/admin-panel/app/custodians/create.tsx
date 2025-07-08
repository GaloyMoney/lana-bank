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
import { Checkbox } from "@lana/web/ui/check-box"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"

import { gql } from "@apollo/client"

import {
  useCustodianCreateMutation,
  type KomainuConfig,
  type CustodianCreateInput,
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

type CustodianType = "komainu"

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

  const [selectedType, setSelectedType] = useState<CustodianType>("komainu")
  const [komainuConfig, setKomainuConfig] = useState<KomainuConfig>({
    name: "",
    apiKey: "",
    apiSecret: "",
    testingInstance: false,
    secretKey: "",
    webhookSecret: "",
  })
  const [error, setError] = useState<string | null>(null)

  const resetForm = () => {
    setSelectedType("komainu")
    setKomainuConfig({
      name: "",
      apiKey: "",
      apiSecret: "",
      testingInstance: false,
      secretKey: "",
      webhookSecret: "",
    })
    setError(null)
  }

  const closeDialog = () => {
    setOpenCreateCustodianDialog(false)
    resetForm()
  }

  const [createCustodian, { loading }] = useCustodianCreateMutation()

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target
    setKomainuConfig((prev) => ({ ...prev, [name]: value }))
  }

  const handleCheckboxChange = (checked: boolean) => {
    setKomainuConfig((prev) => ({ ...prev, testingInstance: checked }))
  }

  const buildCustodianInput = (): CustodianCreateInput => {
    switch (selectedType) {
      case "komainu":
        return { komainu: komainuConfig }
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
              </SelectContent>
            </Select>
          </div>

          {selectedType === "komainu" && (
            <>
              <div>
                <Label htmlFor="name" required>
                  {t("fields.name")}
                </Label>
                <Input
                  id="name"
                  name="name"
                  value={komainuConfig.name}
                  onChange={handleInputChange}
                  placeholder={t("placeholders.name")}
                  required
                  disabled={loading}
                  data-testid="custodian-name-input"
                />
              </div>
              <div>
                <Label htmlFor="apiKey" required>
                  {t("fields.apiKey")}
                </Label>
                <Input
                  id="apiKey"
                  name="apiKey"
                  value={komainuConfig.apiKey}
                  onChange={handleInputChange}
                  placeholder={t("placeholders.apiKey")}
                  required
                  disabled={loading}
                  data-testid="custodian-api-key-input"
                />
              </div>
              <div>
                <Label htmlFor="apiSecret" required>
                  {t("fields.apiSecret")}
                </Label>
                <Input
                  id="apiSecret"
                  name="apiSecret"
                  type="password"
                  value={komainuConfig.apiSecret}
                  onChange={handleInputChange}
                  placeholder={t("placeholders.apiSecret")}
                  required
                  disabled={loading}
                  data-testid="custodian-api-secret-input"
                />
              </div>
              <div>
                <Label htmlFor="secretKey" required>
                  {t("fields.secretKey")}
                </Label>
                <Input
                  id="secretKey"
                  name="secretKey"
                  type="password"
                  value={komainuConfig.secretKey}
                  onChange={handleInputChange}
                  placeholder={t("placeholders.secretKey")}
                  required
                  disabled={loading}
                  data-testid="custodian-secret-key-input"
                />
              </div>
              <div>
                <Label htmlFor="webhookSecret" required>
                  {t("fields.webhookSecret")}
                </Label>
                <Input
                  id="webhookSecret"
                  name="webhookSecret"
                  type="password"
                  value={komainuConfig.webhookSecret}
                  onChange={handleInputChange}
                  placeholder={t("placeholders.webhookSecret")}
                  required
                  disabled={loading}
                  data-testid="custodian-webhook-secret-input"
                />
              </div>

              <div className="flex items-center space-x-2">
                <Checkbox
                  id="testingInstance"
                  checked={komainuConfig.testingInstance}
                  onCheckedChange={handleCheckboxChange}
                  disabled={loading}
                  data-testid="custodian-testing-instance-checkbox"
                />
                <Label htmlFor="testingInstance">{t("fields.testingInstance")}</Label>
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
