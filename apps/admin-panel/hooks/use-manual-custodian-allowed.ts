import { useDomainConfigsQuery } from "@/lib/graphql/generated"

export const useManualCustodianAllowed = (): boolean => {
  const { data } = useDomainConfigsQuery({ variables: { first: 100 } })
  return data?.domainConfigs.nodes.find((c) => c.key === "allow-manual-custodian")?.value === true
}
