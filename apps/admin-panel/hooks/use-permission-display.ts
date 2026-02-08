import { useTranslations } from "next-intl"

export type PermissionTranslation = {
  label: string
  description: string
}

export function usePermissionDisplay() {
  const t = useTranslations("Permissions")

  const getTranslation = (permissionName: string): PermissionTranslation => {
    return {
      label: t(`${permissionName}.label`),
      description: t(`${permissionName}.description`),
    }
  }

  return {
    getTranslation,
  }
}
