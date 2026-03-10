import { useDomainConfigsQuery } from "@/lib/graphql/generated"

export const useManualCollateralEnabled = (): boolean => {
  const { data } = useDomainConfigsQuery({ variables: { first: 100 } })
  return data?.domainConfigs.nodes.find((c) => c.key === "manual-collateral")?.value !== false
}