"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import DataTable, { Column } from "@/components/data-table"
import Balance from "@/components/balance/balance"
import {
  useCreditFacilityLedgerAccountsQuery,
  LedgerAccountInfoFragment,
} from "@/lib/graphql/generated"

gql`
  fragment LedgerAccountInfo on LedgerAccount {
    name
    ledgerAccountId
    normalBalanceType
    balanceRange {
      __typename
      ... on UsdLedgerAccountBalanceRange {
        close {
          usdSettled: settled {
            net
          }
        }
      }
      ... on BtcLedgerAccountBalanceRange {
        close {
          btcSettled: settled {
            net
          }
        }
      }
    }
  }

  query CreditFacilityLedgerAccounts($publicId: PublicId!) {
    creditFacilityByPublicId(id: $publicId) {
      id
      ledgerAccounts {
        facilityAccount {
          ...LedgerAccountInfo
        }
        disbursedReceivableNotYetDueAccount {
          ...LedgerAccountInfo
        }
        disbursedReceivableDueAccount {
          ...LedgerAccountInfo
        }
        disbursedReceivableOverdueAccount {
          ...LedgerAccountInfo
        }
        disbursedDefaultedAccount {
          ...LedgerAccountInfo
        }
        collateralAccount {
          ...LedgerAccountInfo
        }
        proceedsFromLiquidationAccount {
          ...LedgerAccountInfo
        }
        interestReceivableNotYetDueAccount {
          ...LedgerAccountInfo
        }
        interestReceivableDueAccount {
          ...LedgerAccountInfo
        }
        interestReceivableOverdueAccount {
          ...LedgerAccountInfo
        }
        interestDefaultedAccount {
          ...LedgerAccountInfo
        }
        interestIncomeAccount {
          ...LedgerAccountInfo
        }
        feeIncomeAccount {
          ...LedgerAccountInfo
        }
        paymentHoldingAccount {
          ...LedgerAccountInfo
        }
        uncoveredOutstandingAccount {
          ...LedgerAccountInfo
        }
      }
    }
  }
`

interface CreditFacilityLedgerAccountsPageProps {
  params: Promise<{
    "credit-facility-id": string
  }>
}

export default function CreditFacilityLedgerAccountsPage({
  params,
}: CreditFacilityLedgerAccountsPageProps) {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.LedgerAccounts")
  const { "credit-facility-id": publicId } = use(params)

  const { data, loading, error } = useCreditFacilityLedgerAccountsQuery({
    variables: { publicId },
  })

  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacilityByPublicId?.ledgerAccounts) return null
  const { ledgerAccounts } = data.creditFacilityByPublicId

  const ledgerAccountsData: LedgerAccountInfoFragment[] = [
    ledgerAccounts.collateralAccount,
    ledgerAccounts.proceedsFromLiquidationAccount,
    ledgerAccounts.disbursedDefaultedAccount,
    ledgerAccounts.disbursedReceivableDueAccount,
    ledgerAccounts.disbursedReceivableNotYetDueAccount,
    ledgerAccounts.disbursedReceivableOverdueAccount,
    ledgerAccounts.facilityAccount,
    ledgerAccounts.feeIncomeAccount,
    ledgerAccounts.interestDefaultedAccount,
    ledgerAccounts.interestIncomeAccount,
    ledgerAccounts.interestReceivableDueAccount,
    ledgerAccounts.interestReceivableNotYetDueAccount,
    ledgerAccounts.interestReceivableOverdueAccount,
    ledgerAccounts.paymentHoldingAccount,
    ledgerAccounts.uncoveredOutstandingAccount,
  ]

  const columns: Column<LedgerAccountInfoFragment>[] = [
    {
      key: "name",
      header: t("table.headers.name"),
      width: "55%",
    },
    {
      key: "normalBalanceType",
      header: t("table.headers.normalBalanceType"),
      render: (balanceType) => t(`balanceTypes.${balanceType}`),
    },
    {
      key: "balanceRange",
      header: t("table.headers.balance"),
      render: (_, account) => {
        const { balanceRange } = account
        if (balanceRange.__typename === "UsdLedgerAccountBalanceRange") {
          return <Balance amount={balanceRange.close.usdSettled.net} currency="usd" />
        }
        if (balanceRange.__typename === "BtcLedgerAccountBalanceRange") {
          return <Balance amount={balanceRange.close.btcSettled.net} currency="btc" />
        }
        return "-"
      },
    },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <DataTable
          data={ledgerAccountsData}
          columns={columns}
          loading={loading}
          navigateTo={(account) => `/ledger-accounts/${account.ledgerAccountId}`}
        />
      </CardContent>
    </Card>
  )
}
