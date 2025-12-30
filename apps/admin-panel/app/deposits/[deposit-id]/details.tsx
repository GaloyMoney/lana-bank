"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"
import { ArrowRight, ExternalLinkIcon, RotateCcw } from "lucide-react"
import Link from "next/link"

import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import { DepositStatusBadge } from "../status-badge"

import { DepositRevertDialog } from "./revert"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"

import { GetDepositDetailsQuery, DepositStatus } from "@/lib/graphql/generated"

type DepositDetailsProps = {
  deposit: NonNullable<GetDepositDetailsQuery["depositByPublicId"]>
}

const DepositDetailsCard: React.FC<DepositDetailsProps> = ({ deposit }) => {
  const t = useTranslations("Deposits.DepositDetails.DepositDetailsCard")
  const [openDepositRevertDialog, setOpenDepositRevertDialog] = useState<
    GetDepositDetailsQuery["depositByPublicId"] | null
  >(null)

  const details: DetailItemProps[] = [
    {
      label: t("fields.customerEmail"),
      value: deposit.account.customer.email,
      href: `/customers/${deposit.account.customer.publicId}`,
    },
    {
      label: t("fields.depositAmount"),
      value: <Balance amount={deposit.amount} currency="usd" />,
    },
    {
      label: t("fields.depositReference"),
      value: deposit.reference === deposit.depositId ? t("values.na") : deposit.reference,
    },
    {
      label: t("fields.createdAt"),
      value: formatDate(deposit.createdAt),
    },
    {
      label: t("fields.status"),
      value: <DepositStatusBadge status={deposit.status} />,
      valueTestId: "deposit-status-badge",
    },
  ]

  const footerContent = (
    <>
      <Button variant="outline" asChild>
        <Link href={`/deposit-accounts/${deposit.account.publicId}`}>
          {t("buttons.viewDepositAccount")}
          <ArrowRight />
        </Link>
      </Button>
      {deposit.status === DepositStatus.Confirmed && (
        <Button
          data-testid="deposit-revert-button"
          variant="outline"
          onClick={() => setOpenDepositRevertDialog(deposit)}
        >
          <RotateCcw className="h-4 w-4" />
          {t("buttons.revert")}
        </Button>
      )}
      <Button variant="outline" asChild>
        <a
          href={`https://cockpit.sumsub.com/checkus#/kyt/txns?search=${deposit.depositId}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          {t("buttons.viewOnSumsub")}
          <ExternalLinkIcon className="h-4 w-4" />
        </a>
      </Button>
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
      {openDepositRevertDialog && (
        <DepositRevertDialog
          depositData={openDepositRevertDialog}
          openDepositRevertDialog={Boolean(openDepositRevertDialog)}
          setOpenDepositRevertDialog={() => setOpenDepositRevertDialog(null)}
        />
      )}
    </>
  )
}

export default DepositDetailsCard
