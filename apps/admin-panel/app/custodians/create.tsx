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
import { Textarea } from "@lana/web/ui/textarea"
import { Label } from "@lana/web/ui/label"
import { Checkbox } from "@lana/web/ui/checkbox"
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
  type BitgoConfig,
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

type CustodianType = "komainu" | "bitgo"

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
  const [bitgoConfig, setBitgoConfig] = useState<BitgoConfig>({
    name: "",
    longLivedToken: "",
    passphrase: "",
    testingInstance: false,
    enterpriseId: "",
    webhookSecret: "",
    webhookUrl: "",
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
    setBitgoConfig({
      name: "",
      longLivedToken: "",
      passphrase: "",
      testingInstance: false,
      enterpriseId: "",
      webhookSecret: "",
      webhookUrl: "",
    })
    setError(null)
  }

  const closeDialog = () => {
    setOpenCreateCustodianDialog(false)
    resetForm()
  }

  const [createCustodian, { loading }] = useCustodianCreateMutation()

  const handleKomainuInputChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ) => {
    const { name, value } = e.target
    setKomainuConfig((prev) => ({ ...prev, [name]: value }))
  }

  const handleBitgoInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target
    setBitgoConfig((prev) => ({ ...prev, [name]: value }))
  }

  const handleKomainuCheckboxChange = (checked: boolean) => {
    setKomainuConfig((prev) => ({ ...prev, testingInstance: checked }))
  }

  const handleBitgoCheckboxChange = (checked: boolean) => {
    setBitgoConfig((prev) => ({ ...prev, testingInstance: checked }))
  }

  const buildCustodianInput = (): CustodianCreateInput => {
    switch (selectedType) {
      case "komainu":
        return { komainu: komainuConfig }
      case "bitgo":
        return { bitgo: bitgoConfig }
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
                  onChange={handleKomainuInputChange}
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
                  onChange={handleKomainuInputChange}
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
                  onChange={handleKomainuInputChange}
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
                <Textarea
                  id="secretKey"
                  name="secretKey"
                  value={komainuConfig.secretKey}
                  onChange={handleKomainuInputChange}
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
                  onChange={handleKomainuInputChange}
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
                  onCheckedChange={handleKomainuCheckboxChange}
                  disabled={loading}
                  data-testid="custodian-testing-instance-checkbox"
                />
                <Label htmlFor="testingInstance">{t("fields.testingInstance")}</Label>
              </div>
            </>
          )}

          {selectedType === "bitgo" && (
            <>
              <div>
                <Label htmlFor="name" required>
                  {t("fields.name")}
                </Label>
                <Input
                  id="name"
                  name="name"
                  value={bitgoConfig.name}
                  onChange={handleBitgoInputChange}
                  placeholder={t("placeholders.name")}
                  required
                  disabled={loading}
                  data-testid="custodian-name-input"
                />
              </div>
              <div>
                <Label htmlFor="longLivedToken" required>
                  {t("fields.longLivedToken")}
                </Label>
                <Input
                  id="longLivedToken"
                  name="longLivedToken"
                  type="password"
                  value={bitgoConfig.longLivedToken}
                  onChange={handleBitgoInputChange}
                  placeholder={t("placeholders.longLivedToken")}
                  required
                  disabled={loading}
                  data-testid="custodian-long-lived-token-input"
                />
              </div>
              <div>
                <Label htmlFor="passphrase" required>
                  {t("fields.passphrase")}
                </Label>
                <Input
                  id="passphrase"
                  name="passphrase"
                  type="password"
                  value={bitgoConfig.passphrase}
                  onChange={handleBitgoInputChange}
                  placeholder={t("placeholders.passphrase")}
                  required
                  disabled={loading}
                  data-testid="custodian-passphrase-input"
                />
              </div>
              <div>
                <Label htmlFor="enterpriseId" required>
                  {t("fields.enterpriseId")}
                </Label>
                <Input
                  id="enterpriseId"
                  name="enterpriseId"
                  value={bitgoConfig.enterpriseId}
                  onChange={handleBitgoInputChange}
                  placeholder={t("placeholders.enterpriseId")}
                  required
                  disabled={loading}
                  data-testid="custodian-enterprise-id-input"
                />
              </div>
              <div>
                <Label htmlFor="webhookUrl" required>
                  {t("fields.webhookUrl")}
                </Label>
                <Input
                  id="webhookUrl"
                  name="webhookUrl"
                  value={bitgoConfig.webhookUrl}
                  onChange={handleBitgoInputChange}
                  placeholder={t("placeholders.webhookUrl")}
                  required
                  disabled={loading}
                  data-testid="custodian-webhook-url-input"
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
                  value={bitgoConfig.webhookSecret}
                  onChange={handleBitgoInputChange}
                  placeholder={t("placeholders.webhookSecret")}
                  required
                  disabled={loading}
                  data-testid="custodian-webhook-secret-input"
                />
              </div>

              <div className="flex items-center space-x-2">
                <Checkbox
                  id="testingInstance"
                  checked={bitgoConfig.testingInstance}
                  onCheckedChange={handleBitgoCheckboxChange}
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
