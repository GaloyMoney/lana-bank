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
import { ProfitAndLossConfigUpdateDialog } from "./profit-and-loss-config-update"

import { DetailItem } from "@/components/details"
import {
  useDepositConfigQuery,
  useCreditConfigQuery,
  useBalanceSheetConfigQuery,
  useProfitAndLossStatementConfigQuery,
} from "@/lib/graphql/generated"

gql`
  query depositConfig {
    depositConfig {
      chartOfAccountsOmnibusParentCode
      chartOfAccountsIndividualDepositAccountsParentCode
      chartOfAccountsGovernmentEntityDepositAccountsParentCode
      chartOfAccountPrivateCompanyDepositAccountsParentCode
      chartOfAccountBankDepositAccountsParentCode
      chartOfAccountFinancialInstitutionDepositAccountsParentCode
      chartOfAccountNonDomiciledIndividualDepositAccountsParentCode
      chartOfAccountsFrozenIndividualDepositAccountsParentCode
      chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode
      chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode
      chartOfAccountFrozenBankDepositAccountsParentCode
      chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode
      chartOfAccountFrozenNonDomiciledIndividualDepositAccountsParentCode
    }
  }

  query creditConfig {
    creditConfig {
      chartOfAccountFacilityOmnibusParentCode
      chartOfAccountCollateralOmnibusParentCode
      chartOfAccountLiquidationProceedsOmnibusParentCode
      chartOfAccountPaymentsMadeOmnibusParentCode
      chartOfAccountInterestAddedToObligationsOmnibusParentCode
      chartOfAccountUncoveredOutstandingParentCode
      chartOfAccountFacilityParentCode
      chartOfAccountCollateralParentCode
      chartOfAccountCollateralInLiquidationParentCode
      chartOfAccountInterestIncomeParentCode
      chartOfAccountFeeIncomeParentCode
      chartOfAccountPaymentHoldingParentCode
      chartOfAccountShortTermIndividualDisbursedReceivableParentCode
      chartOfAccountShortTermGovernmentEntityDisbursedReceivableParentCode
      chartOfAccountShortTermPrivateCompanyDisbursedReceivableParentCode
      chartOfAccountShortTermBankDisbursedReceivableParentCode
      chartOfAccountShortTermFinancialInstitutionDisbursedReceivableParentCode
      chartOfAccountShortTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode
      chartOfAccountShortTermNonDomiciledCompanyDisbursedReceivableParentCode
      chartOfAccountLongTermIndividualDisbursedReceivableParentCode
      chartOfAccountLongTermGovernmentEntityDisbursedReceivableParentCode
      chartOfAccountLongTermPrivateCompanyDisbursedReceivableParentCode
      chartOfAccountLongTermBankDisbursedReceivableParentCode
      chartOfAccountLongTermFinancialInstitutionDisbursedReceivableParentCode
      chartOfAccountLongTermForeignAgencyOrSubsidiaryDisbursedReceivableParentCode
      chartOfAccountLongTermNonDomiciledCompanyDisbursedReceivableParentCode
      chartOfAccountShortTermIndividualInterestReceivableParentCode
      chartOfAccountShortTermGovernmentEntityInterestReceivableParentCode
      chartOfAccountShortTermPrivateCompanyInterestReceivableParentCode
      chartOfAccountShortTermBankInterestReceivableParentCode
      chartOfAccountShortTermFinancialInstitutionInterestReceivableParentCode
      chartOfAccountShortTermForeignAgencyOrSubsidiaryInterestReceivableParentCode
      chartOfAccountShortTermNonDomiciledCompanyInterestReceivableParentCode
      chartOfAccountLongTermIndividualInterestReceivableParentCode
      chartOfAccountLongTermGovernmentEntityInterestReceivableParentCode
      chartOfAccountLongTermPrivateCompanyInterestReceivableParentCode
      chartOfAccountLongTermBankInterestReceivableParentCode
      chartOfAccountLongTermFinancialInstitutionInterestReceivableParentCode
      chartOfAccountLongTermForeignAgencyOrSubsidiaryInterestReceivableParentCode
      chartOfAccountLongTermNonDomiciledCompanyInterestReceivableParentCode
      chartOfAccountOverdueIndividualDisbursedReceivableParentCode
      chartOfAccountOverdueGovernmentEntityDisbursedReceivableParentCode
      chartOfAccountOverduePrivateCompanyDisbursedReceivableParentCode
      chartOfAccountOverdueBankDisbursedReceivableParentCode
      chartOfAccountOverdueFinancialInstitutionDisbursedReceivableParentCode
      chartOfAccountOverdueForeignAgencyOrSubsidiaryDisbursedReceivableParentCode
      chartOfAccountOverdueNonDomiciledCompanyDisbursedReceivableParentCode
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

  query ProfitAndLossStatementConfig {
    profitAndLossStatementConfig {
      chartOfAccountsRevenueCode
      chartOfAccountsCostOfRevenueCode
      chartOfAccountsExpensesCode
    }
  }
`

const Modules: React.FC = () => {
  const t = useTranslations("Modules")

  const [openDepositConfigUpdateDialog, setOpenDepositConfigUpdateDialog] =
    useState(false)
  const [openCreditConfigUpdateDialog, setOpenCreditConfigUpdateDialog] = useState(false)
  const [openBalanceSheetConfigUpdateDialog, setOpenBalanceSheetConfigUpdateDialog] =
    useState(false)
  const [openProfitAndLossConfigUpdateDialog, setOpenProfitAndLossConfigUpdateDialog] =
    useState(false)

  const { data: depositConfig, loading: depositConfigLoading } = useDepositConfigQuery()
  const { data: creditConfig, loading: creditConfigLoading } = useCreditConfigQuery()
  const { data: balanceSheetConfig, loading: balanceSheetConfigLoading } =
    useBalanceSheetConfigQuery()
  const { data: profitAndLossConfig, loading: profitAndLossConfigLoading } =
    useProfitAndLossStatementConfigQuery()

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
      <ProfitAndLossConfigUpdateDialog
        open={openProfitAndLossConfigUpdateDialog}
        setOpen={setOpenProfitAndLossConfigUpdateDialog}
        profitAndLossConfig={
          profitAndLossConfig?.profitAndLossStatementConfig || undefined
        }
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
                {t("deposit.setTitle")}
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
                {t("credit.setTitle")}
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
                {t("balanceSheet.setTitle")}
              </Button>
            </CardFooter>
          </>
        )}
      </Card>
      <Card className="mt-3">
        <CardHeader>
          <CardTitle>{t("profitAndLoss.title")}</CardTitle>
          <CardDescription>{t("profitAndLoss.description")}</CardDescription>
        </CardHeader>

        <CardContent>
          {profitAndLossConfigLoading ? (
            <LoaderCircle className="animate-spin" />
          ) : profitAndLossConfig?.profitAndLossStatementConfig ? (
            <DetailsGroup>
              {Object.entries(
                profitAndLossConfig?.profitAndLossStatementConfig || {},
              ).map(
                ([key, value]) =>
                  key !== "__typename" && (
                    <DetailItem
                      key={key}
                      label={t(`profitAndLoss.${key}`)}
                      value={value?.replace(/\./g, "")}
                    />
                  ),
              )}
            </DetailsGroup>
          ) : (
            <div>{t("notYetConfigured")}</div>
          )}
        </CardContent>
        {!profitAndLossConfig?.profitAndLossStatementConfig && (
          <>
            <Separator className="mb-4" />
            <CardFooter className="-mb-3 -mt-1 justify-end">
              <Button
                variant="outline"
                onClick={() => setOpenProfitAndLossConfigUpdateDialog(true)}
              >
                <Pencil />
                {t("profitAndLoss.setTitle")}
              </Button>
            </CardFooter>
          </>
        )}
      </Card>
    </>
  )
}

export default Modules
