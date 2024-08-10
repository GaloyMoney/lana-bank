import Link from "next/link"

import { AuthenticatorAssuranceLevel } from "@ory/client"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"

import { RocketIcon } from "@/components/icons"
import { Checkbox } from "@/components/primitive/check-box"
import { Label } from "@/components/primitive/label"
import { BalanceCard } from "@/components/balance-card"
import { LoanCard } from "@/components/loan/recent-loans-card"
import { getMeAndSession } from "@/lib/auth/get-session.ts"
import { currencyConverter, formatCurrency } from "@/lib/utils"
import { KycKybWrapper } from "@/components/sumsub-wrapper"
import { KycLevel } from "@/lib/graphql/generated"

export default async function Home() {
  const getMeAndSessionResponse = await getMeAndSession()

  if (getMeAndSessionResponse instanceof Error) {
    return (
      <Card className="max-w-[70rem] m-auto">
        <CardHeader>
          <CardTitle>Error</CardTitle>
        </CardHeader>
        <CardContent>
          <CardDescription>{getMeAndSessionResponse.message}</CardDescription>
        </CardContent>
      </Card>
    )
  }

  const kycCompleted =
    getMeAndSessionResponse.me?.level === KycLevel.One ||
    getMeAndSessionResponse.me?.level === KycLevel.Two

  const balance = [
    {
      currency: "US Dollar",
      amount: formatCurrency({
        amount: currencyConverter.centsToUsd(
          getMeAndSessionResponse.me?.balance.checking.settled.amount,
        ),
        currency: "USD",
      }),
    },
  ]

  return (
    <main className="max-w-[70rem] m-auto">
      <>
        <OnboardingCard
          twoFactorAuthEnabled={
            getMeAndSessionResponse.session.authenticator_assurance_level ===
            AuthenticatorAssuranceLevel.Aal2 //can we get this from backend api?
          }
          kycCompleted={kycCompleted}
        />
        <div className="flex gap-4 mt-4 items-stretch">
          <BalanceCard balance={balance} />
          <LoanCard />
        </div>
      </>
    </main>
  )
}

const OnboardingCard = ({
  twoFactorAuthEnabled,
  kycCompleted,
}: {
  twoFactorAuthEnabled: boolean
  kycCompleted: boolean
}) => {
  return (
    <Card className="mt-10">
      <CardHeader className="md:pb-0">
        <div className="flex align-middle gap-4">
          <RocketIcon className="hidden md:block w-10 h-10" />
          <div className="flex flex-col gap-2">
            <CardTitle className="mt-2">
              Complete onboarding steps to Initiate a Loan
            </CardTitle>
            <CardDescription>
              Complete the following steps to initiate to complete your onboarding process
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="mt-6">
        <div className="ml-14 flex flex-col gap-4">
          <Link
            data-test-id="enable-2fa-button"
            className="flex gap-2 items-center"
            href="/settings/2fa"
          >
            <Checkbox checked={twoFactorAuthEnabled} />
            <Label className="hover:underline">Enable Two-Factor Authentication </Label>
          </Link>
          <KycKybWrapper kycCompleted={kycCompleted} />
        </div>
      </CardContent>
    </Card>
  )
}
