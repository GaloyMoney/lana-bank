"use client"

import { toast } from "sonner"
import { useTranslations } from "next-intl"

type IDProps = {
  id: string
  type?: string
}

const ID: React.FC<IDProps> = ({ id }) => {
  const t = useTranslations("Common")

  const copyID = () => {
    navigator.clipboard.writeText(id)
    toast.success(t("copiedToClipboard"))
  }

  return (
    <div className="text-sm">
      <span className="text-mono">{id.slice(0, 7)}...</span>
      <span className="text-blue-600 cursor-pointer" onClick={copyID}>
        {t("copyId")}
      </span>
    </div>
  )
}

export { ID }
