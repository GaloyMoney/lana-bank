import { useTranslations } from "next-intl"

export const NotFound = () => {
  const t = useTranslations("Common")
  return <div>{t("notFound")}</div>
}
