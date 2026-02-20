"use client"

import React, { useState, useMemo } from "react"
import { useTranslations } from "next-intl"
import { useRouter } from "next/navigation"
import { ArrowRight, Snowflake, Sun, XCircle } from "lucide-react"

import { Button } from "@lana/web/ui/button"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@lana/web/ui/tooltip"

import { formatDate } from "@lana/web/utils"

import { DepositAccountStatusBadge } from "../status-badge"

import FreezeDepositAccountDialog from "./freeze-deposit-account"
import CloseDepositAccountDialog from "./close-deposit-account"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"

import {
  GetDepositAccountDetailsQuery,
  DepositAccountStatus,
} from "@/lib/graphql/generated"
import UnfreezeDepositAccountDialog from "@/app/deposit-accounts/[deposit-account-id]/unfreeze-deposit-account"

type DepositAccountDetailsProps = {
  depositAccount: NonNullable<GetDepositAccountDetailsQuery["depositAccountByPublicId"]>
}

const DepositAccountDetailsCard: React.FC<DepositAccountDetailsProps> = ({
  depositAccount,
}) => {
  const t = useTranslations(
    "DepositAccounts.DepositAccountDetails.DepositAccountDetailsCard",
  )
  const tClose = useTranslations(
    "DepositAccounts.DepositAccountDetails.closeDepositAccount",
  )
  const router = useRouter()
  const [openFreezeDialog, setOpenFreezeDialog] = useState(false)
  const [openUnfreezeDialog, setOpenUnfreezeDialog] = useState(false)
  const [openCloseDialog, setOpenCloseDialog] = useState(false)

  const isBalanceZero = useMemo(() => {
    return depositAccount.balance.settled === 0 && depositAccount.balance.pending === 0
  }, [depositAccount.balance])

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

  const handleUnfreezeAccount = () => {
    setOpenUnfreezeDialog(true)
  }

  const handleCloseAccount = () => {
    setOpenCloseDialog(true)
  }

  const details: DetailItemProps[] = [
    {
      label: t("fields.customerId"),
      value: depositAccount.customerId,
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
    },
  ]

  const footerContent = (
    <>
      <Button variant="outline" onClick={handleViewLedgerAccount}>
        {t("buttons.viewLedgerAccount")}
        <ArrowRight />
      </Button>
      {depositAccount.status === DepositAccountStatus.Active && (
        <Button variant="outline" onClick={handleFreezeAccount}>
          <Snowflake />
          {t("buttons.freezeDepositAccount")}
        </Button>
      )}
      {depositAccount.status === DepositAccountStatus.Frozen && (
        <Button variant="outline" onClick={handleUnfreezeAccount}>
          <Sun />
          {t("buttons.unfreezeDepositAccount")}
        </Button>
      )}
      {(depositAccount.status === DepositAccountStatus.Active ||
        depositAccount.status === DepositAccountStatus.Inactive) && (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <span>
                <Button
                  variant="destructive"
                  onClick={handleCloseAccount}
                  disabled={!isBalanceZero}
                >
                  <XCircle />
                  {t("buttons.closeDepositAccount")}
                </Button>
              </span>
            </TooltipTrigger>
            {!isBalanceZero && (
              <TooltipContent>
                <p>{tClose("disabledTooltip")}</p>
              </TooltipContent>
            )}
          </Tooltip>
        </TooltipProvider>
      )}
    </>
  )

  return (
    <>
      <DetailsCard title={t("title")} details={details} footerContent={footerContent} />
      <FreezeDepositAccountDialog
        depositAccountId={depositAccount.depositAccountId}
        balance={depositAccount.balance}
        openFreezeDialog={openFreezeDialog}
        setOpenFreezeDialog={setOpenFreezeDialog}
      />
      <UnfreezeDepositAccountDialog
        depositAccountId={depositAccount.depositAccountId}
        openUnfreezeDialog={openUnfreezeDialog}
        setOpenUnfreezeDialog={setOpenUnfreezeDialog}
      />
      <CloseDepositAccountDialog
        depositAccountId={depositAccount.depositAccountId}
        openCloseDialog={openCloseDialog}
        setOpenCloseDialog={setOpenCloseDialog}
      />
    </>
  )
}

export default DepositAccountDetailsCard
