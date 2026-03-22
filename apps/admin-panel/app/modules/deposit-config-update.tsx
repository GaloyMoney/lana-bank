"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import {
  DEPOSIT_CONFIG_FIELDS,
  DEPOSIT_EMPTY_FORM_DATA,
  DEPOSIT_FIELD_GROUPS,
  DepositAccountCategoryKey,
  buildDepositChanges,
  buildDepositFormDataFromConfig,
} from "./deposit-config-fields"
import { ModuleConfigUpdateDialog } from "./module-config-update-dialog"

import {
  DepositAccountConfigDocument,
  DepositModuleConfig,
  DepositAccountModuleConfigureInput,
  useDepositAccountModuleConfigureMutation,
} from "@/lib/graphql/generated"
import {
  type AccountSetOptionBase,
} from "@/app/components/account-set-combobox"

gql`
  mutation DepositAccountModuleConfigure($input: DepositAccountModuleConfigureInput!) {
    depositAccountModuleConfigure(input: $input) {
      depositAccountConfig {
        chartOfAccountsId
        chartOfAccountsOmnibusParentCode
        chartOfAccountsIndividualDepositAccountsParentCode
        chartOfAccountsGovernmentEntityDepositAccountsParentCode
        chartOfAccountsPrivateCompanyDepositAccountsParentCode
        chartOfAccountsBankDepositAccountsParentCode
        chartOfAccountsFinancialInstitutionDepositAccountsParentCode
        chartOfAccountsNonDomiciledCompanyDepositAccountsParentCode
        chartOfAccountsFrozenIndividualDepositAccountsParentCode
        chartOfAccountsFrozenGovernmentEntityDepositAccountsParentCode
        chartOfAccountsFrozenPrivateCompanyDepositAccountsParentCode
        chartOfAccountsFrozenBankDepositAccountsParentCode
        chartOfAccountsFrozenFinancialInstitutionDepositAccountsParentCode
        chartOfAccountsFrozenNonDomiciledCompanyDepositAccountsParentCode
      }
    }
  }
`

type DepositConfigUpdateDialogProps = {
  setOpen: (isOpen: boolean) => void
  open: boolean
  depositModuleConfig?: DepositModuleConfig
  accountSetOptions?: AccountSetOption[]
  accountSetOptionsError?: boolean
}

type AccountSetOption = AccountSetOptionBase & {
  category: DepositAccountCategoryKey
}

export const DepositConfigUpdateDialog: React.FC<DepositConfigUpdateDialogProps> = ({
  open,
  setOpen,
  depositModuleConfig,
  accountSetOptions = [],
  accountSetOptionsError = false,
}) => {
  const t = useTranslations("Modules")

  const [updateDepositConfig, { loading, error, reset }] =
    useDepositAccountModuleConfigureMutation({
      refetchQueries: [DepositAccountConfigDocument],
    })
  const handleSave = async (input: DepositAccountModuleConfigureInput) => {
    await updateDepositConfig({ variables: { input } })
    toast.success(t("deposit.updateSuccess"))
  }

  return (
    <ModuleConfigUpdateDialog
      open={open}
      setOpen={setOpen}
      moduleKey="deposit"
      moduleConfig={depositModuleConfig}
      accountSetOptions={accountSetOptions}
      accountSetOptionsError={accountSetOptionsError}
      fields={DEPOSIT_CONFIG_FIELDS}
      fieldGroups={DEPOSIT_FIELD_GROUPS}
      emptyFormData={DEPOSIT_EMPTY_FORM_DATA}
      buildFormDataFromConfig={buildDepositFormDataFromConfig}
      buildChanges={buildDepositChanges}
      loading={loading}
      errorMessage={error?.message}
      reset={reset}
      onSave={handleSave}
    />
  )
}
