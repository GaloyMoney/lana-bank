import { useDomainConfigsQuery } from "@/lib/graphql/generated"

export const useManualCustodianEnabled = (): boolean => {
  const { data } = useDomainConfigsQuery({ variables: { first: 100 } })
  return data?.domainConfigs.nodes.find((c) => c.key === "enable-manual-custodian")?.value !== false
}