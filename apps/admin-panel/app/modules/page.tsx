"use client"

import { useMemo, useState } from "react"
import { useTranslations } from "next-intl"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "@lana/web/ui/card"
import { gql, useQuery } from "@apollo/client"

import { Button } from "@lana/web/ui/button"
import { Separator } from "@lana/web/ui/separator"
import { LoaderCircle, Pencil } from "lucide-react"

import { DetailsGroup } from "@lana/web/components/details"

import { DepositConfigUpdateDialog } from "./deposit-config-update"
import { CreditConfigUpdateDialog } from "./credit-config-update"

import { DetailItem } from "@/components/details"
import {
  AccountInfo,
  useDepositConfigQuery,
  useCreditConfigQuery,
  useChartAccountingBaseConfigQuery,
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

  query ChartAccountingBaseConfig {
    chartOfAccounts {
      id
      name
      accountingBaseConfig {
        assetsCode
        liabilitiesCode
        equityCode
        equityRetainedEarningsGainCode
        equityRetainedEarningsLossCode
        revenueCode
        costOfRevenueCode
        expensesCode
      }
    }
  }
`

const CREDIT_ACCOUNT_SET_OPTIONS_QUERY = gql`
  query CreditAccountSetOptions {
    asset: accountSetsByCategory(category: ASSET) {
      accountSetId
      code
      name
    }
    liability: accountSetsByCategory(category: LIABILITY) {
      accountSetId
      code
      name
    }
    equity: accountSetsByCategory(category: EQUITY) {
      accountSetId
      code
      name
    }
    revenue: accountSetsByCategory(category: REVENUE) {
      accountSetId
      code
      name
    }
    costOfRevenue: accountSetsByCategory(category: COST_OF_REVENUE) {
      accountSetId
      code
      name
    }
    expenses: accountSetsByCategory(category: EXPENSES) {
      accountSetId
      code
      name
    }
  }
`

type CreditAccountSetOptionsData = {
  asset: AccountInfo[]
  liability: AccountInfo[]
  equity: AccountInfo[]
  revenue: AccountInfo[]
  costOfRevenue: AccountInfo[]
  expenses: AccountInfo[]
}

const Modules: React.FC = () => {
  const t = useTranslations("Modules")

  const [openDepositConfigUpdateDialog, setOpenDepositConfigUpdateDialog] =
    useState(false)
  const [openCreditConfigUpdateDialog, setOpenCreditConfigUpdateDialog] = useState(false)

  const { data: depositConfig, loading: depositConfigLoading } = useDepositConfigQuery()
  const { data: creditConfig, loading: creditConfigLoading } = useCreditConfigQuery()
  const { data: chartData, loading: chartLoading } = useChartAccountingBaseConfigQuery()
  const { data: accountSetOptionsData } = useQuery<CreditAccountSetOptionsData>(
    CREDIT_ACCOUNT_SET_OPTIONS_QUERY,
  )

  const accountingBaseConfig = chartData?.chartOfAccounts?.accountingBaseConfig
  const accountSetOptions = useMemo(() => {
    if (!accountSetOptionsData) return []

    const categoryMap: Array<{
      key: keyof CreditAccountSetOptionsData
      category:
        | "asset"
        | "liability"
        | "equity"
        | "revenue"
        | "costOfRevenue"
        | "expenses"
    }> = [
      { key: "asset", category: "asset" },
      { key: "liability", category: "liability" },
      { key: "equity", category: "equity" },
      { key: "revenue", category: "revenue" },
      { key: "costOfRevenue", category: "costOfRevenue" },
      { key: "expenses", category: "expenses" },
    ]

    return categoryMap.flatMap(({ key, category }) =>
      (accountSetOptionsData[key] || []).map((option) => ({
        code: option.code,
        name: option.name,
        category,
      })),
    )
  }, [accountSetOptionsData])

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
        accountSetOptions={accountSetOptions}
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
          <CardTitle>{t("accountingBaseConfig.title")}</CardTitle>
          <CardDescription>{t("accountingBaseConfig.description")}</CardDescription>
        </CardHeader>

        <CardContent>
          {chartLoading ? (
            <LoaderCircle className="animate-spin" />
          ) : accountingBaseConfig ? (
            <DetailsGroup>
              {Object.entries(accountingBaseConfig).map(
                ([key, value]) =>
                  key !== "__typename" && (
                    <DetailItem
                      key={key}
                      label={t(`accountingBaseConfig.${key}`)}
                      value={value?.replace(/\./g, "")}
                    />
                  ),
              )}
            </DetailsGroup>
          ) : (
            <div>{t("notYetConfigured")}</div>
          )}
        </CardContent>
      </Card>
    </>
  )
}

export default Modules
