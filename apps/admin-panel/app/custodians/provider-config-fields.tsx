"use client"

import { useState } from "react"
import { useTranslations } from "next-intl"

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

import {
  type KomainuConfig,
  type BitgoConfig,
  type SelfCustodyConfig,
  type ManualConfig,
  type CustodianConfigInput,
  SelfCustodyNetwork,
} from "@/lib/graphql/generated"

const INITIAL_KOMAINU: KomainuConfig = {
  name: "",
  apiKey: "",
  apiSecret: "",
  testingInstance: false,
  secretKey: "",
  webhookSecret: "",
}

const INITIAL_BITGO: BitgoConfig = {
  name: "",
  longLivedToken: "",
  passphrase: "",
  testingInstance: false,
  enterpriseId: "",
  webhookSecret: "",
  webhookUrl: "",
}

const INITIAL_SELF_CUSTODY: SelfCustodyConfig = {
  name: "",
  accountXpub: "",
  network: SelfCustodyNetwork.Mainnet,
}

const INITIAL_MANUAL: ManualConfig = {
  name: "",
}

export type CustodianType = "komainu" | "bitgo" | "selfCustody" | "manual"

export interface CustodianConfigFormState {
  komainuConfig: KomainuConfig
  bitgoConfig: BitgoConfig
  selfCustodyConfig: SelfCustodyConfig
  manualConfig: ManualConfig
  resetAll: () => void
  buildConfigInput: (type: CustodianType) => CustodianConfigInput
  buildManualInput: () => ManualConfig
  handlers: {
    komainu: {
      onInputChange: (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => void
      onCheckboxChange: (checked: boolean) => void
    }
    bitgo: {
      onInputChange: (e: React.ChangeEvent<HTMLInputElement>) => void
      onCheckboxChange: (checked: boolean) => void
    }
    selfCustody: {
      onInputChange: (e: React.ChangeEvent<HTMLInputElement>) => void
      onNetworkChange: (value: SelfCustodyNetwork) => void
    }
    manual: {
      onInputChange: (e: React.ChangeEvent<HTMLInputElement>) => void
    }
  }
}

export const useCustodianConfigForm = (): CustodianConfigFormState => {
  const [komainuConfig, setKomainuConfig] = useState<KomainuConfig>({ ...INITIAL_KOMAINU })
  const [bitgoConfig, setBitgoConfig] = useState<BitgoConfig>({ ...INITIAL_BITGO })
  const [selfCustodyConfig, setSelfCustodyConfig] = useState<SelfCustodyConfig>({
    ...INITIAL_SELF_CUSTODY,
  })
  const [manualConfig, setManualConfig] = useState<ManualConfig>({ ...INITIAL_MANUAL })

  const resetAll = () => {
    setKomainuConfig({ ...INITIAL_KOMAINU })
    setBitgoConfig({ ...INITIAL_BITGO })
    setSelfCustodyConfig({ ...INITIAL_SELF_CUSTODY })
    setManualConfig({ ...INITIAL_MANUAL })
  }

  const buildConfigInput = (type: CustodianType): CustodianConfigInput => {
    switch (type) {
      case "komainu":
        return { komainu: komainuConfig }
      case "bitgo":
        return { bitgo: bitgoConfig }
      case "selfCustody":
        return { selfCustody: selfCustodyConfig }
      case "manual":
        throw new Error("Manual custodians do not have a config input")
    }
  }

  const buildManualInput = () => manualConfig

  return {
    komainuConfig,
    bitgoConfig,
    selfCustodyConfig,
    manualConfig,
    resetAll,
    buildConfigInput,
    buildManualInput,
    handlers: {
      komainu: {
        onInputChange: (e) => {
          const { name, value } = e.target
          setKomainuConfig((prev) => ({ ...prev, [name]: value }))
        },
        onCheckboxChange: (checked) => {
          setKomainuConfig((prev) => ({ ...prev, testingInstance: checked }))
        },
      },
      bitgo: {
        onInputChange: (e) => {
          const { name, value } = e.target
          setBitgoConfig((prev) => ({ ...prev, [name]: value }))
        },
        onCheckboxChange: (checked) => {
          setBitgoConfig((prev) => ({ ...prev, testingInstance: checked }))
        },
      },
      selfCustody: {
        onInputChange: (e) => {
          const { name, value } = e.target
          setSelfCustodyConfig((prev) => ({ ...prev, [name]: value }))
        },
        onNetworkChange: (value) => {
          setSelfCustodyConfig((prev) => ({ ...prev, network: value }))
        },
      },
      manual: {
        onInputChange: (e) => {
          const { name, value } = e.target
          setManualConfig((prev) => ({ ...prev, [name]: value }))
        },
      },
    },
  }
}

interface ProviderConfigFieldsProps {
  type: CustodianType
  form: CustodianConfigFormState
  loading: boolean
  testIdPrefix?: string
}

export const ProviderConfigFields: React.FC<ProviderConfigFieldsProps> = ({
  type,
  form,
  loading,
  testIdPrefix,
}) => {
  const tFields = useTranslations("Custodians.create.fields")
  const tPlaceholders = useTranslations("Custodians.create.placeholders")

  if (type === "komainu") {
    return (
      <>
        <div>
          <Label htmlFor="name" required>
            {tFields("name")}
          </Label>
          <Input
            id="name"
            name="name"
            value={form.komainuConfig.name}
            onChange={form.handlers.komainu.onInputChange}
            placeholder={tPlaceholders("name")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-name-input` : undefined}
          />
        </div>
        <div>
          <Label htmlFor="apiKey" required>
            {tFields("apiKey")}
          </Label>
          <Input
            id="apiKey"
            name="apiKey"
            value={form.komainuConfig.apiKey}
            onChange={form.handlers.komainu.onInputChange}
            placeholder={tPlaceholders("apiKey")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-api-key-input` : undefined}
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
            value={form.komainuConfig.apiSecret}
            onChange={form.handlers.komainu.onInputChange}
            placeholder={tPlaceholders("apiSecret")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-api-secret-input` : undefined}
          />
        </div>
        <div>
          <Label htmlFor="secretKey" required>
            {tFields("secretKey")}
          </Label>
          <Textarea
            id="secretKey"
            name="secretKey"
            value={form.komainuConfig.secretKey}
            onChange={form.handlers.komainu.onInputChange}
            placeholder={tPlaceholders("secretKey")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-secret-key-input` : undefined}
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
            value={form.komainuConfig.webhookSecret}
            onChange={form.handlers.komainu.onInputChange}
            placeholder={tPlaceholders("webhookSecret")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-webhook-secret-input` : undefined}
          />
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox
            id="testingInstance"
            checked={form.komainuConfig.testingInstance}
            onCheckedChange={form.handlers.komainu.onCheckboxChange}
            disabled={loading}
            data-testid={
              testIdPrefix ? `${testIdPrefix}-testing-instance-checkbox` : undefined
            }
          />
          <Label htmlFor="testingInstance">{tFields("testingInstance")}</Label>
        </div>
      </>
    )
  }

  if (type === "bitgo") {
    return (
      <>
        <div>
          <Label htmlFor="name" required>
            {tFields("name")}
          </Label>
          <Input
            id="name"
            name="name"
            value={form.bitgoConfig.name}
            onChange={form.handlers.bitgo.onInputChange}
            placeholder={tPlaceholders("name")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-name-input` : undefined}
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
            value={form.bitgoConfig.longLivedToken}
            onChange={form.handlers.bitgo.onInputChange}
            placeholder={tPlaceholders("longLivedToken")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-long-lived-token-input` : undefined}
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
            value={form.bitgoConfig.passphrase}
            onChange={form.handlers.bitgo.onInputChange}
            placeholder={tPlaceholders("passphrase")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-passphrase-input` : undefined}
          />
        </div>
        <div>
          <Label htmlFor="enterpriseId" required>
            {tFields("enterpriseId")}
          </Label>
          <Input
            id="enterpriseId"
            name="enterpriseId"
            value={form.bitgoConfig.enterpriseId}
            onChange={form.handlers.bitgo.onInputChange}
            placeholder={tPlaceholders("enterpriseId")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-enterprise-id-input` : undefined}
          />
        </div>
        <div>
          <Label htmlFor="webhookUrl" required>
            {tFields("webhookUrl")}
          </Label>
          <Input
            id="webhookUrl"
            name="webhookUrl"
            value={form.bitgoConfig.webhookUrl}
            onChange={form.handlers.bitgo.onInputChange}
            placeholder={tPlaceholders("webhookUrl")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-webhook-url-input` : undefined}
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
            value={form.bitgoConfig.webhookSecret}
            onChange={form.handlers.bitgo.onInputChange}
            placeholder={tPlaceholders("webhookSecret")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-webhook-secret-input` : undefined}
          />
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox
            id="testingInstance"
            checked={form.bitgoConfig.testingInstance}
            onCheckedChange={form.handlers.bitgo.onCheckboxChange}
            disabled={loading}
            data-testid={
              testIdPrefix ? `${testIdPrefix}-testing-instance-checkbox` : undefined
            }
          />
          <Label htmlFor="testingInstance">{tFields("testingInstance")}</Label>
        </div>
      </>
    )
  }

  if (type === "selfCustody") {
    return (
      <>
        <div>
          <Label htmlFor="name" required>
            {tFields("name")}
          </Label>
          <Input
            id="name"
            name="name"
            value={form.selfCustodyConfig.name}
            onChange={form.handlers.selfCustody.onInputChange}
            placeholder={tPlaceholders("name")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-name-input` : undefined}
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
            value={form.selfCustodyConfig.accountXpub}
            onChange={form.handlers.selfCustody.onInputChange}
            placeholder={tPlaceholders("accountXpub")}
            required
            disabled={loading}
            data-testid={testIdPrefix ? `${testIdPrefix}-account-xpub-input` : undefined}
          />
        </div>
        <div>
          <Label htmlFor="network">{tFields("network")}</Label>
          <Select
            value={form.selfCustodyConfig.network}
            onValueChange={form.handlers.selfCustody.onNetworkChange}
            disabled={loading}
          >
            <SelectTrigger
              data-testid={testIdPrefix ? `${testIdPrefix}-network-select` : undefined}
            >
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
    )
  }

  if (type === "manual") {
    return (
      <div>
        <Label htmlFor="name" required>
          {tFields("name")}
        </Label>
        <Input
          id="name"
          name="name"
          value={form.manualConfig.name}
          onChange={form.handlers.manual.onInputChange}
          placeholder={tPlaceholders("name")}
          required
          disabled={loading}
          data-testid={testIdPrefix ? `${testIdPrefix}-name-input` : undefined}
        />
      </div>
    )
  }

  return null
}
