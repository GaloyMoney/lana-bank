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

import { DetailsGroup } from "@lana/web/components/details"

import { DepositConfigUpdateDialog } from "./deposit-config-update"
import { CreditConfigUpdateDialog } from "./credit-config-update"
import { BalanceSheetConfigUpdateDialog } from "./balance-sheet-config-update"

import { DetailItem } from "@/components/details"
import {
  useDepositConfigQuery,
  useCreditConfigQuery,
  useBalanceSheetConfigQuery,
} from "@/lib/graphql/generated"

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
      chartOfAccountInterestReceivableParentCode
      chartOfAccountInterestIncomeParentCode
      chartOfAccountFeeIncomeParentCode
      chartOfAccountIndividualDisbursedReceivableParentCode
      chartOfAccountGovernmentEntityDisbursedReceivableParentCode
      chartOfAccountPrivateCompanyDisbursedReceivableParentCode
      chartOfAccountBankDisbursedReceivableParentCode
      chartOfAccountFinancialInstitutionDisbursedReceivableParentCode
      chartOfAccountForeignAgencyOrSubsidiaryDisbursedReceivableParentCode
      chartOfAccountNonDomiciledCompanyDisbursedReceivableParentCode
    }
  }

  query BalanceSheetConfig {
    balanceSheetConfig {
      chartOfAccountsAssetsCode
      chartOfAccountsLiabilitiesCode
      chartOfAccountsEquityCode
      chartOfAccountsRevenueCode
      chartOfAccountsCostOfRevenueCode
      chartOfAccountsExpensesCode
    }
  }
`

const Modules: React.FC = () => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")

  const [openDepositConfigUpdateDialog, setOpenDepositConfigUpdateDialog] =
    useState(false)
  const [openCreditConfigUpdateDialog, setOpenCreditConfigUpdateDialog] = useState(false)
  const [openBalanceSheetConfigUpdateDialog, setOpenBalanceSheetConfigUpdateDialog] =
    useState(false)

  const { data: depositConfig, loading: depositConfigLoading } = useDepositConfigQuery()
  const { data: creditConfig, loading: creditConfigLoading } = useCreditConfigQuery()
  const { data: balanceSheetConfig, loading: balanceSheetConfigLoading } =
    useBalanceSheetConfigQuery()

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
      <BalanceSheetConfigUpdateDialog
        open={openBalanceSheetConfigUpdateDialog}
        setOpen={setOpenBalanceSheetConfigUpdateDialog}
        balanceSheetConfig={balanceSheetConfig?.balanceSheetConfig || undefined}
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
            <DetailsGroup>
              {Object.entries(depositConfig?.depositConfig || {}).map(
                ([key, value]) =>
                  key !== "__typename" && (
                    <DetailItem
                      key={key}
                      label={t(`deposit.${key}`)}
                      value={value?.replace(/\./g, "")}
                    />
                  ),
              )}
            </DetailsGroup>
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
            <DetailsGroup>
              {Object.entries(creditConfig?.creditConfig || {}).map(
                ([key, value]) =>
                  key !== "__typename" && (
                    <DetailItem
                      key={key}
                      label={t(`credit.${key}`)}
                      value={value?.replace(/\./g, "")}
                    />
                  ),
              )}
            </DetailsGroup>
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
      <Card className="mt-3">
        <CardHeader>
          <CardTitle>{t("balanceSheet.title")}</CardTitle>
          <CardDescription>{t("balanceSheet.description")}</CardDescription>
        </CardHeader>

        <CardContent>
          {balanceSheetConfigLoading ? (
            <LoaderCircle className="animate-spin" />
          ) : balanceSheetConfig?.balanceSheetConfig ? (
            <DetailsGroup>
              {Object.entries(balanceSheetConfig?.balanceSheetConfig || {}).map(
                ([key, value]) =>
                  key !== "__typename" && (
                    <DetailItem
                      key={key}
                      label={t(`balanceSheet.${key}`)}
                      value={value?.replace(/\./g, "")}
                    />
                  ),
              )}
            </DetailsGroup>
          ) : (
            <div>{t("notYetConfigured")}</div>
          )}
        </CardContent>
        {!balanceSheetConfig?.balanceSheetConfig && (
          <>
            <Separator className="mb-4" />
            <CardFooter className="-mb-3 -mt-1 justify-end">
              <Button
                variant="outline"
                onClick={() => setOpenBalanceSheetConfigUpdateDialog(true)}
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
