import { useState } from "react"

import {
  type KomainuConfig,
  type BitgoConfig,
  type SelfCustodyConfig,
  SelfCustodyNetwork,
} from "@/lib/graphql/generated"

const getInitialKomainuConfig = (): KomainuConfig => ({
  name: "",
  apiKey: "",
  apiSecret: "",
  testingInstance: false,
  secretKey: "",
  webhookSecret: "",
})

const getInitialBitgoConfig = (): BitgoConfig => ({
  name: "",
  longLivedToken: "",
  passphrase: "",
  testingInstance: false,
  enterpriseId: "",
  webhookSecret: "",
  webhookUrl: "",
})

const getInitialSelfCustodyConfig = (): SelfCustodyConfig => ({
  name: "",
  accountXpub: "",
  network: SelfCustodyNetwork.Mainnet,
})

export const useCustodianFormState = () => {
  const [komainuConfig, setKomainuConfig] = useState<KomainuConfig>(getInitialKomainuConfig())
  const [bitgoConfig, setBitgoConfig] = useState<BitgoConfig>(getInitialBitgoConfig())
  const [selfCustodyConfig, setSelfCustodyConfig] = useState<SelfCustodyConfig>(
    getInitialSelfCustodyConfig(),
  )

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

  const handleSelfCustodyNetworkChange = (value: SelfCustodyNetwork) => {
    setSelfCustodyConfig((prev) => ({ ...prev, network: value }))
  }

  const resetProviderConfigs = () => {
    setKomainuConfig(getInitialKomainuConfig())
    setBitgoConfig(getInitialBitgoConfig())
    setSelfCustodyConfig(getInitialSelfCustodyConfig())
  }

  return {
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
  }
}
