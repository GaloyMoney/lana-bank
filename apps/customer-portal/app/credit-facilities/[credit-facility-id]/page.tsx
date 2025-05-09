import { gql } from "@apollo/client"

import { DetailItemProps, DetailsCard } from "@lana/web/components/details"

import Link from "next/link"

import { ArrowDownUp, ArrowLeft, Banknote, Clock } from "lucide-react"

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@lana/web/ui/tab"

import FacilityCard from "./facility-card"

import CollateralCard from "./collateral-card"

import TermsCard from "./terms-card"

import { CreditFacilityHistory } from "./history"

import { CreditFacilityDisbursals } from "./disbursals"

import { CreditFacilityRepaymentPlan } from "./repayment-plan"

import { LoanAndCreditFacilityStatusBadge } from "@/app/credit-facility"

import { getCreditFacility } from "@/lib/graphql/query/get-cf"
import { removeUnderscore } from "@/lib/kratos/utils"
import { formatDate } from "@/lib/utils"
import { meQuery } from "@/lib/graphql/query/me"

gql`
  query GetCreditFacility($id: UUID!) {
    creditFacility(id: $id) {
      id
      creditFacilityId
      facilityAmount
      collateralizationState
      status
      createdAt
      activatedAt
      maturesAt
      disbursals {
        id
        disbursalId
        amount
        status
        createdAt
      }
      creditFacilityTerms {
        annualRate
        accrualCycleInterval
        accrualInterval
        oneTimeFeeRate
        duration {
          period
          units
        }
        liquidationCvl
        marginCallCvl
        initialCvl
      }
      balance {
        facilityRemaining {
          usdBalance
        }
        disbursed {
          total {
            usdBalance
          }
          outstanding {
            usdBalance
          }
          dueOutstanding {
            usdBalance
          }
        }
        interest {
          total {
            usdBalance
          }
          outstanding {
            usdBalance
          }
          dueOutstanding {
            usdBalance
          }
        }
        collateral {
          btcBalance
        }
        dueOutstanding {
          usdBalance
        }
        outstanding {
          usdBalance
        }
      }
      currentCvl {
        total
        disbursed
      }
      repaymentPlan {
        repaymentType
        status
        initial
        outstanding
        accrualAt
        dueAt
      }
      history {
        ... on CreditFacilityIncrementalPayment {
          cents
          recordedAt
          txId
        }
        ... on CreditFacilityCollateralUpdated {
          satoshis
          recordedAt
          action
          txId
        }
        ... on CreditFacilityOrigination {
          cents
          recordedAt
          txId
        }
        ... on CreditFacilityCollateralizationUpdated {
          state
          collateral
          outstandingInterest
          outstandingDisbursal
          recordedAt
          price
        }
        ... on CreditFacilityDisbursalExecuted {
          cents
          recordedAt
          txId
        }
        ... on CreditFacilityInterestAccrued {
          cents
          recordedAt
          txId
          days
        }
      }
    }
  }
`

async function page({ params }: { params: Promise<{ "credit-facility-id": string }> }) {
  const { "credit-facility-id": id } = await params
  const [cfData, meData] = await Promise.all([getCreditFacility({ id }), meQuery()])

  if (!cfData || cfData instanceof Error || !cfData.creditFacility) {
    return <div>Not found</div>
  }

  if (!meData || meData instanceof Error) {
    return <div>Error loading customer data</div>
  }

  const details: DetailItemProps[] = [
    {
      label: "Customer Type",
      value: removeUnderscore(meData.me.customer.customerType),
    },
    {
      label: "Issue Date",
      value: formatDate(cfData.creditFacility.createdAt),
    },
    {
      label: "Maturity Date",
      value: formatDate(cfData.creditFacility.maturesAt) || "N/A",
    },
    {
      label: "Current Monthly Payment",
      value: cfData.creditFacility.repaymentPlan?.length ? (
        <div className="flex items-center">
          $
          {(
            Number(cfData.creditFacility.repaymentPlan[0].outstanding) / 100
          ).toLocaleString(undefined, {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2,
          })}
          <span className="text-sm text-muted-foreground ml-2">
            Due {formatDate(cfData.creditFacility.repaymentPlan[0].dueAt)}
          </span>
        </div>
      ) : (
        "–"
      ),
    },
    {
      label: "Interest Calculation Days",
      value: (
        <div className="flex items-center">
          <span>365</span>
          <span className="text-sm text-muted-foreground ml-2">
            Fixed number used for all interest calculations
          </span>
        </div>
      ),
    },
    {
      label: "Collateralization State",
      value: removeUnderscore(cfData.creditFacility.collateralizationState),
    },
    {
      label: "Status",
      value: (
        <LoanAndCreditFacilityStatusBadge
          data-testid="credit-facility-status-badge"
          status={cfData.creditFacility.status}
        />
      ),
    },
  ]

  return (
    <main className="px-2 flex flex-col gap-2 max-w-7xl mx-auto">
      <DetailsCard
        title={
          <div className="flex items-center gap-2">
            <Link href="/">
              <ArrowLeft className="h-4 w-4" />
            </Link>
            <span className="text-md font-semibold">Credit Facility</span>
          </div>
        }
        details={details}
      />
      <div className="flex flex-col gap-2 md:flex-row">
        <FacilityCard data={cfData.creditFacility} />
        <CollateralCard data={cfData.creditFacility} />
      </div>
      <TermsCard data={cfData.creditFacility} />
      <Tabs defaultValue="history" className="w-full">
        <TabsList className="flex h-12 w-full items-center rounded-lg bg-muted p-1">
          <TabsTrigger
            value="history"
            className="flex h-full flex-1 items-center justify-center gap-2 rounded-md data-[state=active]:bg-background data-[state=active]:text-primary"
          >
            <ArrowDownUp className="h-4 w-4" />
            History
          </TabsTrigger>
          <TabsTrigger
            value="repayments"
            className="flex h-full flex-1 items-center justify-center gap-2 rounded-md data-[state=active]:bg-background data-[state=active]:text-primary"
          >
            <Clock className="h-4 w-4" />
            Repayment Plan
          </TabsTrigger>
          <TabsTrigger
            value="disbursals"
            className="flex h-full flex-1 items-center justify-center gap-2 rounded-md data-[state=active]:bg-background data-[state=active]:text-primary"
          >
            <Banknote className="h-4 w-4" />
            Disbursals
          </TabsTrigger>
        </TabsList>

        <TabsContent value="history" className="mt-2">
          <CreditFacilityHistory creditFacility={cfData.creditFacility} />
        </TabsContent>
        <TabsContent value="repayments" className="mt-2">
          <CreditFacilityRepaymentPlan creditFacility={cfData.creditFacility} />
        </TabsContent>
        <TabsContent value="disbursals" className="mt-2">
          <CreditFacilityDisbursals creditFacility={cfData.creditFacility} />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default page
