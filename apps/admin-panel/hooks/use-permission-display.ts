import { useTranslations } from "next-intl"

export type PermissionTranslation = {
  label: string
  description: string
}

export function usePermissionDisplay() {
  const t = useTranslations("Permissions")

  const getTranslation = (permissionName: string): PermissionTranslation => {
    switch (permissionName) {
      case "access_viewer":
        return {
          label: t("access_viewer.label"),
          description: t("access_viewer.description"),
        }
      case "access_writer":
        return {
          label: t("access_writer.label"),
          description: t("access_writer.description"),
        }
      case "accounting_viewer":
        return {
          label: t("accounting_viewer.label"),
          description: t("accounting_viewer.description"),
        }
      case "accounting_writer":
        return {
          label: t("accounting_writer.label"),
          description: t("accounting_writer.description"),
        }
      case "audit_viewer":
        return {
          label: t("audit_viewer.label"),
          description: t("audit_viewer.description"),
        }
      case "contract_creation":
        return {
          label: t("contract_creation.label"),
          description: t("contract_creation.description"),
        }
      case "credit_viewer":
        return {
          label: t("credit_viewer.label"),
          description: t("credit_viewer.description"),
        }
      case "credit_writer":
        return {
          label: t("credit_writer.label"),
          description: t("credit_writer.description"),
        }
      case "collection_payment_date":
        return {
          label: t("collection_payment_date.label"),
          description: t("collection_payment_date.description"),
        }
      case "collection_viewer":
        return {
          label: t("collection_viewer.label"),
          description: t("collection_viewer.description"),
        }
      case "collection_writer":
        return {
          label: t("collection_writer.label"),
          description: t("collection_writer.description"),
        }
      case "credit_term_templates_viewer":
        return {
          label: t("credit_term_templates_viewer.label"),
          description: t("credit_term_templates_viewer.description"),
        }
      case "credit_term_templates_writer":
        return {
          label: t("credit_term_templates_writer.label"),
          description: t("credit_term_templates_writer.description"),
        }
      case "customer_viewer":
        return {
          label: t("customer_viewer.label"),
          description: t("customer_viewer.description"),
        }
      case "customer_writer":
        return {
          label: t("customer_writer.label"),
          description: t("customer_writer.description"),
        }
      case "dashboard_viewer":
        return {
          label: t("dashboard_viewer.label"),
          description: t("dashboard_viewer.description"),
        }
      case "deposit_viewer":
        return {
          label: t("deposit_viewer.label"),
          description: t("deposit_viewer.description"),
        }
      case "deposit_writer":
        return {
          label: t("deposit_writer.label"),
          description: t("deposit_writer.description"),
        }
      case "deposit_freeze":
        return {
          label: t("deposit_freeze.label"),
          description: t("deposit_freeze.description"),
        }
      case "deposit_unfreeze":
        return {
          label: t("deposit_unfreeze.label"),
          description: t("deposit_unfreeze.description"),
        }
      case "exposed_config_viewer":
        return {
          label: t("exposed_config_viewer.label"),
          description: t("exposed_config_viewer.description"),
        }
      case "exposed_config_writer":
        return {
          label: t("exposed_config_writer.label"),
          description: t("exposed_config_writer.description"),
        }
      case "governance_viewer":
        return {
          label: t("governance_viewer.label"),
          description: t("governance_viewer.description"),
        }
      case "governance_writer":
        return {
          label: t("governance_writer.label"),
          description: t("governance_writer.description"),
        }
      case "custody_viewer":
        return {
          label: t("custody_viewer.label"),
          description: t("custody_viewer.description"),
        }
      case "custody_writer":
        return {
          label: t("custody_writer.label"),
          description: t("custody_writer.description"),
        }
      case "report_viewer":
        return {
          label: t("report_viewer.label"),
          description: t("report_viewer.description"),
        }
      case "report_writer":
        return {
          label: t("report_writer.label"),
          description: t("report_writer.description"),
        }
      default:
        return { label: permissionName, description: "" }
    }
  }

  return {
    getTranslation,
  }
}
