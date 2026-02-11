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
import { CreditAccountCategoryKey } from "./credit-config-fields"

import { DetailItem } from "@/components/details"
import {
  AccountInfo,
  useDepositConfigQuery,
  useCreditConfigQuery,
  useChartAccountingBaseConfigQuery,
  useCreditAccountSetOptionsQuery,
  type CreditAccountSetOptionsQuery,
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
      chartOfAccountNonDomiciledCompanyDepositAccountsParentCode
      chartOfAccountsFrozenIndividualDepositAccountsParentCode
      chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode
      chartOfAccountFrozenPrivateCompanyDepositAccountsParentCode
      chartOfAccountFrozenBankDepositAccountsParentCode
      chartOfAccountFrozenFinancialInstitutionDepositAccountsParentCode
      chartOfAccountFrozenNonDomiciledCompanyDepositAccountsParentCode
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
      chartOfAccountLiquidatedCollateralParentCode
      chartOfAccountProceedsFromLiquidationParentCode
      chartOfAccountInterestIncomeParentCode
      chartOfAccountFeeIncomeParentCode
      chartOfAccountPaymentHoldingParentCode
      chartOfAccountDisbursedDefaultedParentCode
      chartOfAccountInterestDefaultedParentCode
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

gql`
  query CreditAccountSetOptions {
    offBalanceSheet: descendantAccountSetsByCategory(category: OFF_BALANCE_SHEET) {
      accountSetId
      code
      name
    }
    asset: descendantAccountSetsByCategory(category: ASSET) {
      accountSetId
      code
      name
    }
    liability: descendantAccountSetsByCategory(category: LIABILITY) {
      accountSetId
      code
      name
    }
    equity: descendantAccountSetsByCategory(category: EQUITY) {
      accountSetId
      code
      name
    }
    revenue: descendantAccountSetsByCategory(category: REVENUE) {
      accountSetId
      code
      name
    }
    costOfRevenue: descendantAccountSetsByCategory(category: COST_OF_REVENUE) {
      accountSetId
      code
      name
    }
    expenses: descendantAccountSetsByCategory(category: EXPENSES) {
      accountSetId
      code
      name
    }
  }
`

const Modules: React.FC = () => {
  const t = useTranslations("Modules")

  const [openDepositConfigUpdateDialog, setOpenDepositConfigUpdateDialog] =
    useState(false)
  const [openCreditConfigUpdateDialog, setOpenCreditConfigUpdateDialog] = useState(false)

  const { data: depositConfig, loading: depositConfigLoading } = useDepositConfigQuery()
  const { data: creditConfig, loading: creditConfigLoading } = useCreditConfigQuery()
  const { data: chartData, loading: chartLoading } = useChartAccountingBaseConfigQuery()
  const { data: accountSetOptionsData, error: accountSetOptionsError } =
    useCreditAccountSetOptionsQuery()

  const accountingBaseConfig = chartData?.chartOfAccounts?.accountingBaseConfig
  const accountSetOptions = useMemo(() => {
    if (!accountSetOptionsData) return []

    type AccountSetOptionsKey = Extract<
      keyof CreditAccountSetOptionsQuery,
      CreditAccountCategoryKey
    >
    const categoryKeys: AccountSetOptionsKey[] = [
      "offBalanceSheet",
      "asset",
      "liability",
      "equity",
      "revenue",
      "costOfRevenue",
      "expenses",
    ]

    return categoryKeys.flatMap((category) => {
      const options = accountSetOptionsData[category] ?? []

      return options.map((option) => ({
        accountSetId: option.accountSetId,
        code: option.code,
        name: option.name,
        category,
      }))
    })
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
        accountSetOptionsError={Boolean(accountSetOptionsError)}
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
