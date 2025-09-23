"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"
import { useRouter } from "next/navigation"
import { Snowflake, ArrowRight } from "lucide-react"

import { Badge } from "@lana/web/ui/badge"
import { Button } from "@lana/web/ui/button"

import FreezeDepositAccountDialog from "./freeze-deposit-account"

import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"
import {
  DepositAccountStatus,
  GetCustomerBasicDetailsQuery,
} from "@/lib/graphql/generated"

type DepositAccountProps = {
  balance: NonNullable<
    NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>["depositAccount"]
  >["balance"]
  publicId: string
  status: DepositAccountStatus
  depositAccountId: string
  ledgerAccounts: NonNullable<
    NonNullable<GetCustomerBasicDetailsQuery["customerByPublicId"]>["depositAccount"]
  >["ledgerAccounts"]
}

export const DepositAccount: React.FC<DepositAccountProps> = ({
  balance,
  publicId,
  status,
  depositAccountId,
  ledgerAccounts,
}) => {
  const t = useTranslations("Customers.CustomerDetails.depositAccount")
  const router = useRouter()
  const [openFreezeDialog, setOpenFreezeDialog] = useState(false)

  const handleViewLedgerAccount = () => {
    const accountId =
      status === DepositAccountStatus.Frozen
        ? ledgerAccounts?.frozenDepositAccountId
        : ledgerAccounts?.depositAccountId

    if (accountId) {
      router.push(`/ledger-accounts/${accountId}`)
    }
  }

  const handleFreezeAccount = () => {
    setOpenFreezeDialog(true)
  }

  const details: DetailItemProps[] = [
    {
      label: t("labels.checkingSettled"),
      value: <Balance amount={balance.settled} currency="usd" />,
    },
    {
      label: t("labels.pendingWithdrawals"),
      value: <Balance amount={balance.pending} currency="usd" />,
    },
    {
      label: t("labels.status"),
      value: <DepositAccountStatusBadge status={status} />,
    },
  ]

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        columns={3}
        className="w-full md:w-3/4"
        publicId={publicId}
        footerContent={
          <>
            <Button variant="outline" onClick={handleViewLedgerAccount}>
              {t("buttons.viewLedgerAccount")}
              <ArrowRight />
            </Button>
            {status !== DepositAccountStatus.Frozen && (
              <Button variant="outline" onClick={handleFreezeAccount}>
                <Snowflake />
                {t("buttons.freezeDepositAccount")}
              </Button>
            )}
          </>
        }
      />
      <FreezeDepositAccountDialog
        depositAccountId={depositAccountId}
        balance={balance}
        openFreezeDialog={openFreezeDialog}
        setOpenFreezeDialog={setOpenFreezeDialog}
      />
    </>
  )
}

export const DepositAccountStatusBadge: React.FC<{ status: DepositAccountStatus }> = ({
  status,
}) => {
  const t = useTranslations("Customers.CustomerDetails.depositAccount.status")

  const getVariant = (status: DepositAccountStatus) => {
    switch (status) {
      case DepositAccountStatus.Active:
        return "success"
      case DepositAccountStatus.Frozen:
        return "destructive"
      case DepositAccountStatus.Inactive:
        return "secondary"
      default: {
        const exhaustiveCheck: never = status
        return exhaustiveCheck
      }
    }
  }

  return <Badge variant={getVariant(status)}>{t(status.toLowerCase())}</Badge>
}
