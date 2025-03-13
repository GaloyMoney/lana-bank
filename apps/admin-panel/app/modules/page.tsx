"use client"

import { useState } from "react"
import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "@lana/web/ui/card"
import { gql } from "@apollo/client"

import { Button } from "@lana/web/ui/button"
import { Separator } from "@lana/web/ui/separator"
import { LoaderCircle, Pencil } from "lucide-react"

import { DepositConfigUpdateDialog } from "./deposit-config-update"

import { CreditConfigUpdateDialog } from "./credit-config-update"

import { DetailItem } from "@/components/details"
import { useDepositConfigQuery, useCreditConfigQuery } from "@/lib/graphql/generated"

gql`
  query depositConfig {
    depositConfig {
      chartOfAccountsDepositAccountsParentCode
      chartOfAccountsOmnibusParentCode
    }
  }

  query creditConfig {
    creditConfig {
      chartOfAccountFacilityOmnibusParentCode
      chartOfAccountCollateralOmnibusParentCode
      chartOfAccountFacilityParentCode
      chartOfAccountCollateralParentCode
      chartOfAccountDisbursedReceivableParentCode
      chartOfAccountInterestReceivableParentCode
      chartOfAccountInterestIncomeParentCode
      chartOfAccountFeeIncomeParentCode
    }
  }
`

const Modules: React.FC = () => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [openDepositConfigUpdateDialog, setOpenDepositConfigUpdateDialog] =
    useState(false)
  const [openCreditConfigUpdateDialog, setOpenCreditConfigUpdateDialog] = useState(false)

  const { data: depositConfig, loading: depositConfigLoading } = useDepositConfigQuery()
  const { data: creditConfig, loading: creditConfigLoading } = useCreditConfigQuery()

  return (
    <>
      <DepositConfigUpdateDialog
        open={openDepositConfigUpdateDialog}
        setOpen={setOpenDepositConfigUpdateDialog}
        depositModuleConfig={depositConfig?.depositConfig || undefined}
      />
      <CreditConfigUpdateDialog
        open={openCreditConfigUpdateDialog}
        setOpen={setOpenCreditConfigUpdateDialog}
        creditModuleConfig={creditConfig?.creditConfig || undefined}
      />
      <Card>
        <CardHeader>
          <CardTitle>{t("deposit.title")}</CardTitle>
          <CardDescription>{t("deposit.description")}</CardDescription>
        </CardHeader>

        <CardContent>
          {depositConfigLoading ? (
            <LoaderCircle className="animate-spin" />
          ) : depositConfig?.depositConfig ? (
            <>
              {Object.entries(depositConfig?.depositConfig || {}).map(([key, value]) => (
                <DetailItem key={key} label={t(`deposit.${key}`)} value={value} />
              ))}
            </>
          ) : (
            <div>{t("notYetConfigured")}</div>
          )}
        </CardContent>
        {!depositConfig?.depositConfig && (
          <>
            {" "}
            <Separator className="mb-4" />
            <CardFooter className="-mb-3 -mt-1 justify-end">
              <Button
                variant="outline"
                onClick={() => setOpenDepositConfigUpdateDialog(true)}
              >
                <Pencil />
                {tCommon("set")}
              </Button>
            </CardFooter>
          </>
        )}
      </Card>
      <Card className="mt-3">
        <CardHeader>
          <CardTitle>{t("credit.title")}</CardTitle>
          <CardDescription>{t("credit.description")}</CardDescription>
        </CardHeader>

        <CardContent>
          {creditConfigLoading ? (
            <LoaderCircle className="animate-spin" />
          ) : creditConfig?.creditConfig ? (
            <>
              {Object.entries(creditConfig?.creditConfig || {}).map(([key, value]) => (
                <DetailItem key={key} label={t(`credit.${key}`)} value={value} />
              ))}
            </>
          ) : (
            <div>{t("notYetConfigured")}</div>
          )}
        </CardContent>
        {!creditConfig?.creditConfig && (
          <>
            <Separator className="mb-4" />
            <CardFooter className="-mb-3 -mt-1 justify-end">
              <Button
                variant="outline"
                onClick={() => setOpenCreditConfigUpdateDialog(true)}
              >
                <Pencil />
                {tCommon("set")}
              </Button>
            </CardFooter>
          </>
        )}
      </Card>
    </>
  )
}

export default Modules
