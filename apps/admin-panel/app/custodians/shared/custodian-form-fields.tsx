"use client"

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
  SelfCustodyNetwork,
} from "@/lib/graphql/generated"

interface KomainuFormFieldsProps {
  config: KomainuConfig
  onInputChange: (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => void
  onCheckboxChange: (checked: boolean) => void
  loading: boolean
  tFields: (key: string) => string
  tPlaceholders: (key: string) => string
  dataTestId?: boolean
}

export const KomainuFormFields: React.FC<KomainuFormFieldsProps> = ({
  config,
  onInputChange,
  onCheckboxChange,
  loading,
  tFields,
  tPlaceholders,
  dataTestId = false,
}) => (
  <>
    <div>
      <Label htmlFor="name" required>
        {tFields("name")}
      </Label>
      <Input
        id="name"
        name="name"
        value={config.name}
        onChange={onInputChange}
        placeholder={tPlaceholders("name")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-name-input" })}
      />
    </div>
    <div>
      <Label htmlFor="apiKey" required>
        {tFields("apiKey")}
      </Label>
      <Input
        id="apiKey"
        name="apiKey"
        value={config.apiKey}
        onChange={onInputChange}
        placeholder={tPlaceholders("apiKey")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-api-key-input" })}
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
        value={config.apiSecret}
        onChange={onInputChange}
        placeholder={tPlaceholders("apiSecret")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-api-secret-input" })}
      />
    </div>
    <div>
      <Label htmlFor="secretKey" required>
        {tFields("secretKey")}
      </Label>
      <Textarea
        id="secretKey"
        name="secretKey"
        value={config.secretKey}
        onChange={onInputChange}
        placeholder={tPlaceholders("secretKey")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-secret-key-input" })}
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
        value={config.webhookSecret}
        onChange={onInputChange}
        placeholder={tPlaceholders("webhookSecret")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-webhook-secret-input" })}
      />
    </div>
    <div className="flex items-center space-x-2">
      <Checkbox
        id="testingInstance"
        checked={config.testingInstance}
        onCheckedChange={onCheckboxChange}
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-testing-instance-checkbox" })}
      />
      <Label htmlFor="testingInstance">{tFields("testingInstance")}</Label>
    </div>
  </>
)

interface BitgoFormFieldsProps {
  config: BitgoConfig
  onInputChange: (e: React.ChangeEvent<HTMLInputElement>) => void
  onCheckboxChange: (checked: boolean) => void
  loading: boolean
  tFields: (key: string) => string
  tPlaceholders: (key: string) => string
  dataTestId?: boolean
}

export const BitgoFormFields: React.FC<BitgoFormFieldsProps> = ({
  config,
  onInputChange,
  onCheckboxChange,
  loading,
  tFields,
  tPlaceholders,
  dataTestId = false,
}) => (
  <>
    <div>
      <Label htmlFor="name" required>
        {tFields("name")}
      </Label>
      <Input
        id="name"
        name="name"
        value={config.name}
        onChange={onInputChange}
        placeholder={tPlaceholders("name")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-name-input" })}
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
        value={config.longLivedToken}
        onChange={onInputChange}
        placeholder={tPlaceholders("longLivedToken")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-long-lived-token-input" })}
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
        value={config.passphrase}
        onChange={onInputChange}
        placeholder={tPlaceholders("passphrase")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-passphrase-input" })}
      />
    </div>
    <div>
      <Label htmlFor="enterpriseId" required>
        {tFields("enterpriseId")}
      </Label>
      <Input
        id="enterpriseId"
        name="enterpriseId"
        value={config.enterpriseId}
        onChange={onInputChange}
        placeholder={tPlaceholders("enterpriseId")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-enterprise-id-input" })}
      />
    </div>
    <div>
      <Label htmlFor="webhookUrl" required>
        {tFields("webhookUrl")}
      </Label>
      <Input
        id="webhookUrl"
        name="webhookUrl"
        value={config.webhookUrl}
        onChange={onInputChange}
        placeholder={tPlaceholders("webhookUrl")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-webhook-url-input" })}
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
        value={config.webhookSecret}
        onChange={onInputChange}
        placeholder={tPlaceholders("webhookSecret")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-webhook-secret-input" })}
      />
    </div>
    <div className="flex items-center space-x-2">
      <Checkbox
        id="testingInstance"
        checked={config.testingInstance}
        onCheckedChange={onCheckboxChange}
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-testing-instance-checkbox" })}
      />
      <Label htmlFor="testingInstance">{tFields("testingInstance")}</Label>
    </div>
  </>
)

interface SelfCustodyFormFieldsProps {
  config: SelfCustodyConfig
  onInputChange: (e: React.ChangeEvent<HTMLInputElement>) => void
  onNetworkChange: (value: SelfCustodyNetwork) => void
  loading: boolean
  tFields: (key: string) => string
  tPlaceholders: (key: string) => string
  dataTestId?: boolean
}

export const SelfCustodyFormFields: React.FC<SelfCustodyFormFieldsProps> = ({
  config,
  onInputChange,
  onNetworkChange,
  loading,
  tFields,
  tPlaceholders,
  dataTestId = false,
}) => (
  <>
    <div>
      <Label htmlFor="name" required>
        {tFields("name")}
      </Label>
      <Input
        id="name"
        name="name"
        value={config.name}
        onChange={onInputChange}
        placeholder={tPlaceholders("name")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-name-input" })}
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
        value={config.accountXpub}
        onChange={onInputChange}
        placeholder={tPlaceholders("accountXpub")}
        required
        disabled={loading}
        {...(dataTestId && { "data-testid": "custodian-account-xpub-input" })}
      />
    </div>
    <div>
      <Label htmlFor="network">{tFields("network")}</Label>
      <Select value={config.network} onValueChange={onNetworkChange} disabled={loading}>
        <SelectTrigger {...(dataTestId && { "data-testid": "custodian-network-select" })}>
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
