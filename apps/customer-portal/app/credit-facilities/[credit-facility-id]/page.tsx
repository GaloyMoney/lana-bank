import { gql } from "@apollo/client"

import { DetailItemProps, DetailsCard } from "@lana/web/components/details"

import Link from "next/link"

import { ArrowDownUp, ArrowLeft, Banknote, Clock } from "lucide-react"

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@lana/web/ui/tab"

import { formatDate } from "@lana/web/utils"

import FacilityCard from "./facility-card"

import CollateralCard from "./collateral-card"

import TermsCard from "./terms-card"

import { CreditFacilityHistory } from "./history"

import { CreditFacilityDisbursals } from "./disbursals"

import { CreditFacilityRepaymentPlan } from "./repayment-plan"

import { LoanAndCreditFacilityStatusBadge } from "@/app/credit-facility"

import { getCreditFacility } from "@/lib/graphql/query/get-cf"
import { removeUnderscore } from "@/lib/utils"
import { GetCreditFacilityQuery } from "@/lib/graphql/generated"

gql`
  query GetCreditFacility($id: UUID!) {
    creditFacility(id: $id) {
      id
      creditFacilityId
      facilityAmount
      collateralizationState
      status
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
      currentCvl
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
          effective
        }
        ... on CreditFacilityCollateralUpdated {
          satoshis
          recordedAt
          action
          txId
          effective
        }
        ... on CreditFacilityApproved {
          cents
          recordedAt
          txId
          effective
        }
        ... on CreditFacilityCollateralizationUpdated {
          state
          collateral
          outstandingInterest
          outstandingDisbursal
          recordedAt
          price
          effective
        }
        ... on CreditFacilityDisbursalExecuted {
          cents
          recordedAt
          txId
          effective
        }
        ... on CreditFacilityInterestAccrued {
          cents
          recordedAt
          txId
          days
          effective
        }
        ... on PendingCreditFacilityCollateralizationUpdated {
          pendingState: state
          collateral
          price
          recordedAt
          effective
        }
      }
    }
  }
`

type CreditFacilityData = NonNullable<GetCreditFacilityQuery["creditFacility"]>

async function page({ params }: { params: Promise<{ "credit-facility-id": string }> }) {
  const { "credit-facility-id": id } = await params
  const data = await getCreditFacility({
    id,
  })

  if (!data || data instanceof Error || !data.creditFacility) {
    return <div>Not found</div>
  }

  const creditFacility: CreditFacilityData = data.creditFacility

  const details: DetailItemProps[] = [
    {
      label: "Activated At",
      value: formatDate(creditFacility.activatedAt),
    },
    {
      label: "Collateralization State",
      value: removeUnderscore(creditFacility.collateralizationState),
    },
    {
      label: "Matures At",
      value: creditFacility.maturesAt
        ? formatDate(creditFacility.maturesAt)
        : "N/A",
    },
    {
      label: "Status",
      value: (
        <LoanAndCreditFacilityStatusBadge
          data-testid="credit-facility-status-badge"
          status={creditFacility.status}
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
        <FacilityCard data={data.creditFacility} />
        <CollateralCard data={data.creditFacility} />
      </div>
      <TermsCard data={data.creditFacility} />
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
          <CreditFacilityHistory creditFacility={data.creditFacility} />
        </TabsContent>
        <TabsContent value="repayments" className="mt-2">
          <CreditFacilityRepaymentPlan creditFacility={data.creditFacility} />
        </TabsContent>
        <TabsContent value="disbursals" className="mt-2">
          <CreditFacilityDisbursals creditFacility={data.creditFacility} />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default page
