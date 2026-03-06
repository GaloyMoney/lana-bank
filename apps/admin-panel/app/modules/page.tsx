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
import { gql } from "@apollo/client"

import { Button } from "@lana/web/ui/button"
import { Separator } from "@lana/web/ui/separator"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@lana/web/ui/table"
import { LoaderCircle, Pencil } from "lucide-react"


import { DepositConfigUpdateDialog } from "./deposit-config-update"
import { CreditConfigUpdateDialog } from "./credit-config-update"
import {
  DEPOSIT_CONFIG_FIELDS,
  DEPOSIT_FIELD_GROUPS,
  DepositAccountCategoryKey,
  type DepositConfigField,
} from "./deposit-config-fields"
import {
  CREDIT_CONFIG_FIELDS,
  CREDIT_FIELD_GROUPS,
  CreditAccountCategoryKey,
  type CreditConfigField,
} from "./credit-config-fields"

import {
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

type ConfigField = DepositConfigField | CreditConfigField

interface ConfigGroupedDisplayProps {
  fields: ConfigField[]
  groups: { key: string; titleKey: string }[]
  config: object
  accountSetOptions: { accountSetId: string; code: string; name: string; category: string }[]
  moduleKey: string
}

const ConfigGroupedDisplay: React.FC<ConfigGroupedDisplayProps> = ({
  fields,
  groups,
  config,
  accountSetOptions,
  moduleKey,
}) => {
  const t = useTranslations("Modules")
  const configEntries = Object.fromEntries(
    Object.entries(config).filter(([k]) => k !== "__typename"),
  )

  return (
    <div className="space-y-4">
      {groups.map((group) => {
        const groupFields = fields.filter((field) => field.group === group.key)
        if (groupFields.length === 0) return null
        return (
          <div
            key={group.key}
            className="space-y-3 rounded-lg border border-border bg-muted/30 p-4"
          >
            <div className="text-sm font-semibold">
              {t(`${moduleKey}.groups.${group.titleKey}`)}
            </div>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("tableHeaders.group")}</TableHead>
                  <TableHead>{t("tableHeaders.chartParent")}</TableHead>
                  <TableHead>{t("tableHeaders.category")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {groupFields.map((field) => {
                  const rawValue = (configEntries[field.key] as string) ?? ""
                  const optionsForCategory = accountSetOptions.filter(
                    (o) => o.category === field.category,
                  )
                  const matchedOption = rawValue
                    ? optionsForCategory.find((o) => o.code === rawValue)
                    : null
                  const displayName = matchedOption?.name ?? "\u2014"
                  const displayCode = rawValue || "\u2014"

                  return (
                    <TableRow key={field.key}>
                      <TableCell className="font-medium">
                        {t(`${moduleKey}.${field.key}`)}
                      </TableCell>
                      <TableCell>
                        <div>{displayName}</div>
                        <div className="font-mono text-xs text-muted-foreground">
                          {displayCode}
                        </div>
                      </TableCell>
                      <TableCell className="text-muted-foreground">
                        {t(`accountCategories.${field.category}`)}
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          </div>
        )
      })}
    </div>
  )
}

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
  const depositAccountSetOptions = useMemo(
    () =>
      accountSetOptions.filter(
        (
          option,
        ): option is (typeof accountSetOptions)[number] & {
          category: DepositAccountCategoryKey
        } =>
          option.category === "asset" || option.category === "liability",
      ),
    [accountSetOptions],
  )

  return (
    <>
      <DepositConfigUpdateDialog
        open={openDepositConfigUpdateDialog}
        setOpen={setOpenDepositConfigUpdateDialog}
        depositModuleConfig={depositConfig?.depositConfig || undefined}
        accountSetOptions={depositAccountSetOptions}
        accountSetOptionsError={Boolean(accountSetOptionsError)}
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
            <ConfigGroupedDisplay
              fields={DEPOSIT_CONFIG_FIELDS}
              groups={DEPOSIT_FIELD_GROUPS}
              config={depositConfig.depositConfig}
              accountSetOptions={depositAccountSetOptions}
              moduleKey="deposit"
            />
          ) : (
            <div>{t("notYetConfigured")}</div>
          )}
        </CardContent>
        <>
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
            <ConfigGroupedDisplay
              fields={CREDIT_CONFIG_FIELDS}
              groups={CREDIT_FIELD_GROUPS}
              config={creditConfig.creditConfig}
              accountSetOptions={accountSetOptions}
              moduleKey="credit"
            />
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
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
              {Object.entries(accountingBaseConfig).map(
                ([key, value]) =>
                  key !== "__typename" && (
                    <div key={key} className="space-y-1 rounded-lg border border-border p-3">
                      <div className="text-xs text-muted-foreground">
                        {t(`accountingBaseConfig.${key}`)}
                      </div>
                      <div className="font-mono text-sm">
                        {value || "\u2014"}
                      </div>
                    </div>
                  ),
              )}
            </div>
          ) : (
            <div>{t("notYetConfigured")}</div>
          )}
        </CardContent>
      </Card>
    </>
  )
}

export default Modules
