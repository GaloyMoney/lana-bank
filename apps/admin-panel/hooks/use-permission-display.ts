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
      case PermissionSetName.AppViewer:
        return {
          label: t("app_viewer.label"),
          description: t("app_viewer.description"),
        }
      case PermissionSetName.AppWriter:
        return {
          label: t("app_writer.label"),
          description: t("app_writer.description"),
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
    }

    const exhaustiveCheck: never = permissionName
    return exhaustiveCheck
  }

  return {
    getTranslation,
  }
}
