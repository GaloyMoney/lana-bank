"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import {
  CREDIT_CONFIG_FIELDS,
  CREDIT_EMPTY_FORM_DATA,
  CREDIT_FIELD_GROUPS,
  CreditAccountCategoryKey,
  buildCreditChanges,
  buildCreditFormDataFromConfig,
} from "./credit-config-fields"
import { ModuleConfigUpdateDialog } from "./module-config-update-dialog"

import {
  CreditConfigDocument,
  CreditModuleConfig,
  CreditModuleConfigureInput,
  useCreditModuleConfigureMutation,
} from "@/lib/graphql/generated"
import {
  type AccountSetOptionBase,
} from "@/app/components/account-set-combobox"

gql`
  mutation CreditModuleConfigure($input: CreditModuleConfigureInput!) {
    creditModuleConfigure(input: $input) {
      creditConfig {
        chartOfAccountsId
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
  }
`

type AccountSetOption = AccountSetOptionBase & {
  category: CreditAccountCategoryKey
}

type CreditConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  creditModuleConfig?: CreditModuleConfig
  accountSetOptions?: AccountSetOption[]
  accountSetOptionsError?: boolean
}


export const CreditConfigUpdateDialog: React.FC<CreditConfigUpdateDialogProps> = ({
  open,
  setOpen,
  creditModuleConfig,
  accountSetOptions = [],
  accountSetOptionsError = false,
}) => {
  const t = useTranslations("Modules")

  const [updateCreditConfig, { loading, error, reset }] =
    useCreditModuleConfigureMutation({
      refetchQueries: [CreditConfigDocument],
    })
  const handleSave = async (input: CreditModuleConfigureInput) => {
    await updateCreditConfig({ variables: { input } })
    toast.success(t("credit.updateSuccess"))
  }

  return (
    <ModuleConfigUpdateDialog
      open={open}
      setOpen={setOpen}
      moduleKey="credit"
      moduleConfig={creditModuleConfig}
      accountSetOptions={accountSetOptions}
      accountSetOptionsError={accountSetOptionsError}
      fields={CREDIT_CONFIG_FIELDS}
      fieldGroups={CREDIT_FIELD_GROUPS}
      emptyFormData={CREDIT_EMPTY_FORM_DATA}
      buildFormDataFromConfig={buildCreditFormDataFromConfig}
      buildChanges={buildCreditChanges}
      loading={loading}
      errorMessage={error?.message}
      reset={reset}
      onSave={handleSave}
    />
  )
}
