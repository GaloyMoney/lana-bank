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
  useCustodianConfigUpdateMutation,
  type KomainuConfig,
  type BitgoConfig,
  type SelfCustodyConfig,
  type CustodianConfigInput,
  SelfCustodyNetwork,
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
  const [selfCustodyConfig, setSelfCustodyConfig] = useState<SelfCustodyConfig>({
    name: "",
    accountXpub: "",
    network: SelfCustodyNetwork.Mainnet,
  })
  const [error, setError] = useState<string | null>(null)

  const resetForm = () => {
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
    setSelfCustodyConfig({
      name: "",
      accountXpub: "",
      network: SelfCustodyNetwork.Mainnet,
    })
    setError(null)
  }

  const closeDialog = () => {
    setOpen(false)
    resetForm()
  }

  const [updateConfig, { loading }] = useCustodianConfigUpdateMutation()

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

  const handleSelfCustodyInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target
    setSelfCustodyConfig((prev) => ({ ...prev, [name]: value }))
  }

  const handleKomainuCheckboxChange = (checked: boolean) => {
    setKomainuConfig((prev) => ({ ...prev, testingInstance: checked }))
  }

  const handleBitgoCheckboxChange = (checked: boolean) => {
    setBitgoConfig((prev) => ({ ...prev, testingInstance: checked }))
  }

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
            <>
              <div>
                <Label htmlFor="name" required>
                  {tFields("name")}
                </Label>
                <Input
                  id="name"
                  name="name"
                  value={komainuConfig.name}
                  onChange={handleKomainuInputChange}
                  placeholder={tPlaceholders("name")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="apiKey" required>
                  {tFields("apiKey")}
                </Label>
                <Input
                  id="apiKey"
                  name="apiKey"
                  value={komainuConfig.apiKey}
                  onChange={handleKomainuInputChange}
                  placeholder={tPlaceholders("apiKey")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="apiSecret" required>
                  {tFields("apiSecret")}
                </Label>
                <Input
                  id="apiSecret"
                  name="apiSecret"
                  type="password"
                  value={komainuConfig.apiSecret}
                  onChange={handleKomainuInputChange}
                  placeholder={tPlaceholders("apiSecret")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="secretKey" required>
                  {tFields("secretKey")}
                </Label>
                <Textarea
                  id="secretKey"
                  name="secretKey"
                  value={komainuConfig.secretKey}
                  onChange={handleKomainuInputChange}
                  placeholder={tPlaceholders("secretKey")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="webhookSecret" required>
                  {tFields("webhookSecret")}
                </Label>
                <Input
                  id="webhookSecret"
                  name="webhookSecret"
                  type="password"
                  value={komainuConfig.webhookSecret}
                  onChange={handleKomainuInputChange}
                  placeholder={tPlaceholders("webhookSecret")}
                  required
                  disabled={loading}
                />
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="testingInstance"
                  checked={komainuConfig.testingInstance}
                  onCheckedChange={handleKomainuCheckboxChange}
                  disabled={loading}
                />
                <Label htmlFor="testingInstance">{tFields("testingInstance")}</Label>
              </div>
            </>
          )}

          {providerType === "bitgo" && (
            <>
              <div>
                <Label htmlFor="name" required>
                  {tFields("name")}
                </Label>
                <Input
                  id="name"
                  name="name"
                  value={bitgoConfig.name}
                  onChange={handleBitgoInputChange}
                  placeholder={tPlaceholders("name")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="longLivedToken" required>
                  {tFields("longLivedToken")}
                </Label>
                <Input
                  id="longLivedToken"
                  name="longLivedToken"
                  type="password"
                  value={bitgoConfig.longLivedToken}
                  onChange={handleBitgoInputChange}
                  placeholder={tPlaceholders("longLivedToken")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="passphrase" required>
                  {tFields("passphrase")}
                </Label>
                <Input
                  id="passphrase"
                  name="passphrase"
                  type="password"
                  value={bitgoConfig.passphrase}
                  onChange={handleBitgoInputChange}
                  placeholder={tPlaceholders("passphrase")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="enterpriseId" required>
                  {tFields("enterpriseId")}
                </Label>
                <Input
                  id="enterpriseId"
                  name="enterpriseId"
                  value={bitgoConfig.enterpriseId}
                  onChange={handleBitgoInputChange}
                  placeholder={tPlaceholders("enterpriseId")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="webhookUrl" required>
                  {tFields("webhookUrl")}
                </Label>
                <Input
                  id="webhookUrl"
                  name="webhookUrl"
                  value={bitgoConfig.webhookUrl}
                  onChange={handleBitgoInputChange}
                  placeholder={tPlaceholders("webhookUrl")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="webhookSecret" required>
                  {tFields("webhookSecret")}
                </Label>
                <Input
                  id="webhookSecret"
                  name="webhookSecret"
                  type="password"
                  value={bitgoConfig.webhookSecret}
                  onChange={handleBitgoInputChange}
                  placeholder={tPlaceholders("webhookSecret")}
                  required
                  disabled={loading}
                />
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="testingInstance"
                  checked={bitgoConfig.testingInstance}
                  onCheckedChange={handleBitgoCheckboxChange}
                  disabled={loading}
                />
                <Label htmlFor="testingInstance">{tFields("testingInstance")}</Label>
              </div>
            </>
          )}

          {providerType === "selfCustody" && (
            <>
              <div>
                <Label htmlFor="name" required>
                  {tFields("name")}
                </Label>
                <Input
                  id="name"
                  name="name"
                  value={selfCustodyConfig.name}
                  onChange={handleSelfCustodyInputChange}
                  placeholder={tPlaceholders("name")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="accountXpub" required>
                  {tFields("accountXpub")}
                </Label>
                <Input
                  id="accountXpub"
                  name="accountXpub"
                  type="password"
                  value={selfCustodyConfig.accountXpub}
                  onChange={handleSelfCustodyInputChange}
                  placeholder={tPlaceholders("accountXpub")}
                  required
                  disabled={loading}
                />
              </div>
              <div>
                <Label htmlFor="network">{tFields("network")}</Label>
                <Select
                  value={selfCustodyConfig.network}
                  onValueChange={(value: SelfCustodyNetwork) =>
                    setSelfCustodyConfig((prev) => ({ ...prev, network: value }))
                  }
                  disabled={loading}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value={SelfCustodyNetwork.Mainnet}>Mainnet</SelectItem>
                    <SelectItem value={SelfCustodyNetwork.Testnet3}>Testnet3</SelectItem>
                    <SelectItem value={SelfCustodyNetwork.Testnet4}>Testnet4</SelectItem>
                    <SelectItem value={SelfCustodyNetwork.Signet}>Signet</SelectItem>
                  </SelectContent>
                </Select>
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
