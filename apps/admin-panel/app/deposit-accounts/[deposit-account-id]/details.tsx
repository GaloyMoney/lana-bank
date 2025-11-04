"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"
import { useRouter } from "next/navigation"
import { ArrowRight, Snowflake } from "lucide-react"

import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import { DepositAccountStatusBadge } from "../status-badge"

import FreezeDepositAccountDialog from "./freeze-deposit-account"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"

import {
  GetDepositAccountDetailsQuery,
  DepositAccountStatus,
} from "@/lib/graphql/generated"

type DepositAccountDetailsProps = {
  depositAccount: NonNullable<GetDepositAccountDetailsQuery["depositAccountByPublicId"]>
}

const DepositAccountDetailsCard: React.FC<DepositAccountDetailsProps> = ({
  depositAccount,
}) => {
  const t = useTranslations("DepositAccounts.DepositAccountDetails.DepositAccountDetailsCard")
  const router = useRouter()
  const [openFreezeDialog, setOpenFreezeDialog] = useState(false)

  const handleViewLedgerAccount = () => {
    const accountId =
      depositAccount.status === DepositAccountStatus.Frozen
        ? depositAccount.ledgerAccounts?.frozenDepositAccountId
        : depositAccount.ledgerAccounts?.depositAccountId

    if (accountId) {
      router.push(`/ledger-accounts/${accountId}`)
    }
  }

  const handleFreezeAccount = () => {
    setOpenFreezeDialog(true)
  }

  const details: DetailItemProps[] = [
    {
      label: t("fields.customerEmail"),
      value: depositAccount.customer.email,
      href: `/customers/${depositAccount.customer.publicId}`,
    },
    {
      label: t("fields.settledBalance"),
      value: <Balance amount={depositAccount.balance.settled} currency="usd" />,
    },
    {
      label: t("fields.pendingBalance"),
      value: <Balance amount={depositAccount.balance.pending} currency="usd" />,
    },
    {
      label: t("fields.createdAt"),
      value: formatDate(depositAccount.createdAt),
    },
    {
      label: t("fields.status"),
      value: <DepositAccountStatusBadge status={depositAccount.status} />,
      valueTestId: "deposit-account-status-badge",
    },
  ]

  const footerContent = (
    <>
      <Button variant="outline" onClick={handleViewLedgerAccount}>
        {t("buttons.viewLedgerAccount")}
        <ArrowRight />
      </Button>
      {depositAccount.status !== DepositAccountStatus.Frozen && (
        <Button variant="outline" onClick={handleFreezeAccount}>
          <Snowflake />
          {t("buttons.freezeDepositAccount")}
        </Button>
      )}
    </>
  )

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        footerContent={footerContent}
        className="max-w-7xl m-auto"
      />
      <FreezeDepositAccountDialog
        depositAccountId={depositAccount.depositAccountId}
        balance={depositAccount.balance}
        openFreezeDialog={openFreezeDialog}
        setOpenFreezeDialog={setOpenFreezeDialog}
      />
    </>
  )
}

export default DepositAccountDetailsCard
