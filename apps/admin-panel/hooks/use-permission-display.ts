import { useTranslations } from "next-intl"

import { PermissionSetName } from "@/lib/graphql/generated"

export type PermissionTranslation = {
  label: string
  description: string
}

export function usePermissionDisplay() {
  const t = useTranslations("Permissions")

  const getTranslation = (permissionName: PermissionSetName): PermissionTranslation => {
    switch (permissionName) {
      case PermissionSetName.AccessViewer:
        return {
          label: t("access_viewer.label"),
          description: t("access_viewer.description"),
        }
      case PermissionSetName.AccessWriter:
        return {
          label: t("access_writer.label"),
          description: t("access_writer.description"),
        }
      case PermissionSetName.AccountingViewer:
        return {
          label: t("accounting_viewer.label"),
          description: t("accounting_viewer.description"),
        }
      case PermissionSetName.AccountingWriter:
        return {
          label: t("accounting_writer.label"),
          description: t("accounting_writer.description"),
        }
      case PermissionSetName.AuditViewer:
        return {
          label: t("audit_viewer.label"),
          description: t("audit_viewer.description"),
        }
      case PermissionSetName.ContractCreation:
        return {
          label: t("contract_creation.label"),
          description: t("contract_creation.description"),
        }
      case PermissionSetName.CreditViewer:
        return {
          label: t("credit_viewer.label"),
          description: t("credit_viewer.description"),
        }
      case PermissionSetName.CreditWriter:
        return {
          label: t("credit_writer.label"),
          description: t("credit_writer.description"),
        }
      case PermissionSetName.CreditPaymentDate:
        return {
          label: t("credit_payment_date.label"),
          description: t("credit_payment_date.description"),
        }
      case PermissionSetName.CreditTermTemplatesViewer:
        return {
          label: t("credit_term_templates_viewer.label"),
          description: t("credit_term_templates_viewer.description"),
        }
      case PermissionSetName.CreditTermTemplatesWriter:
        return {
          label: t("credit_term_templates_writer.label"),
          description: t("credit_term_templates_writer.description"),
        }
      case PermissionSetName.CustomerViewer:
        return {
          label: t("customer_viewer.label"),
          description: t("customer_viewer.description"),
        }
      case PermissionSetName.CustomerWriter:
        return {
          label: t("customer_writer.label"),
          description: t("customer_writer.description"),
        }
      case PermissionSetName.DashboardViewer:
        return {
          label: t("dashboard_viewer.label"),
          description: t("dashboard_viewer.description"),
        }
      case PermissionSetName.DepositViewer:
        return {
          label: t("deposit_viewer.label"),
          description: t("deposit_viewer.description"),
        }
      case PermissionSetName.DepositWriter:
        return {
          label: t("deposit_writer.label"),
          description: t("deposit_writer.description"),
        }
      case PermissionSetName.DepositFreeze:
        return {
          label: t("deposit_freeze.label"),
          description: t("deposit_freeze.description"),
        }
      case PermissionSetName.DepositUnfreeze:
        return {
          label: t("deposit_unfreeze.label"),
          description: t("deposit_unfreeze.description"),
        }
      case PermissionSetName.ExposedConfigViewer:
        return {
          label: t("exposed_config_viewer.label"),
          description: t("exposed_config_viewer.description"),
        }
      case PermissionSetName.ExposedConfigWriter:
        return {
          label: t("exposed_config_writer.label"),
          description: t("exposed_config_writer.description"),
        }
      case PermissionSetName.GovernanceViewer:
        return {
          label: t("governance_viewer.label"),
          description: t("governance_viewer.description"),
        }
      case PermissionSetName.GovernanceWriter:
        return {
          label: t("governance_writer.label"),
          description: t("governance_writer.description"),
        }
      case PermissionSetName.CustodyViewer:
        return {
          label: t("custody_viewer.label"),
          description: t("custody_viewer.description"),
        }
      case PermissionSetName.CustodyWriter:
        return {
          label: t("custody_writer.label"),
          description: t("custody_writer.description"),
        }
      case PermissionSetName.ReportViewer:
        return {
          label: t("report_viewer.label"),
          description: t("report_viewer.description"),
        }
      case PermissionSetName.ReportWriter:
        return {
          label: t("report_writer.label"),
          description: t("report_writer.description"),
        }
    }

    const exhaustiveCheck: never = permissionName
    return exhaustiveCheck
  }

  return {
    getTranslation,
  }
}
