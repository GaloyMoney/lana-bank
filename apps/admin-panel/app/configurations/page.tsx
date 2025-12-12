"use client"

import { useEffect, useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { LoaderCircle } from "lucide-react"
import { toast } from "sonner"

import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Button } from "@lana/web/ui/button"
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"

import {
  NotificationEmailConfigDocument,
  useNotificationEmailConfigQuery,
  useNotificationEmailConfigUpdateMutation,
} from "@/lib/graphql/generated"

gql`
  query notificationEmailConfig {
    notificationEmailConfig {
      fromEmail
      fromName
    }
  }
`

gql`
  mutation notificationEmailConfigUpdate($input: NotificationEmailConfigInput!) {
    notificationEmailConfigUpdate(input: $input) {
      notificationEmailConfig {
        fromEmail
        fromName
      }
    }
  }
`

export default function ConfigurationsPage() {
  const t = useTranslations("Configurations")

  const [senderForm, setSenderForm] = useState({ fromEmail: "", fromName: "" })

  const {
    data: senderConfig,
    loading: senderConfigLoading,
  } = useNotificationEmailConfigQuery()

  const senderConfigData = senderConfig?.notificationEmailConfig

  const [updateSenderConfig, { loading: senderConfigSaving }] =
    useNotificationEmailConfigUpdateMutation({
      refetchQueries: [NotificationEmailConfigDocument],
    })

  const isBusy = senderConfigLoading || senderConfigSaving

  useEffect(() => {
    setSenderForm({
      fromEmail: senderConfigData?.fromEmail || "",
      fromName: senderConfigData?.fromName || "",
    })
  }, [senderConfigData?.fromEmail, senderConfigData?.fromName])

  const handleSenderSave = async () => {
    try {
      const result = await updateSenderConfig({
        variables: {
          input: {
            fromEmail: senderForm.fromEmail,
            fromName: senderForm.fromName,
          },
        },
      })

      const updatedConfig = result.data?.notificationEmailConfigUpdate.notificationEmailConfig

      if (!updatedConfig) {
        toast.error(t("notificationEmail.saveError"))
        return
      }

      toast.success(t("notificationEmail.saveSuccess"))
      setSenderForm({
        fromEmail: updatedConfig.fromEmail,
        fromName: updatedConfig.fromName, 
      })
    } catch (error) {
      console.error("Failed to update notification email configuration:", error)

      const errorMessage = error instanceof Error ? error.message : null

      toast.error(
        errorMessage
          ? t("notificationEmail.saveErrorWithReason", { error: errorMessage })
          : t("notificationEmail.saveError"),
      )
    }
  }

  const handleReset = () => {
    setSenderForm({
      fromEmail: senderConfigData?.fromEmail || "",
      fromName: senderConfigData?.fromName || "",
    })
  }

  return (
    <div className="space-y-3">
      <Card>
        <CardHeader>
          <CardTitle>{t("notificationEmail.title")}</CardTitle>
          <CardDescription>{t("notificationEmail.description")}</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4">
          {senderConfigLoading ? (
            <LoaderCircle className="animate-spin" />
          ) : (
            <>
              <div className="grid gap-2">
                <Label htmlFor="fromEmail">{t("notificationEmail.fromEmail")}</Label>
                <Input
                  id="fromEmail"
                  type="email"
                  value={senderForm.fromEmail}
                  disabled={isBusy}
                  onChange={(e) =>
                    setSenderForm((prev) => ({ ...prev, fromEmail: e.target.value }))
                  }
                  required
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="fromName">{t("notificationEmail.fromName")}</Label>
                <Input
                  id="fromName"
                  value={senderForm.fromName}
                  disabled={isBusy}
                  required
                  onChange={(e) =>
                    setSenderForm((prev) => ({ ...prev, fromName: e.target.value }))
                  }
                />
              </div>
            </>
          )}
        </CardContent>
        <CardFooter className="justify-end gap-2">
          <Button
            variant="outline"
            onClick={handleReset}
            disabled={isBusy}
          >
            {t("notificationEmail.reset")}
          </Button>
          <Button
            onClick={handleSenderSave}
            disabled={
              isBusy ||
              senderForm.fromEmail.trim().length === 0 ||
              senderForm.fromName.trim().length === 0
            }
          >
            {senderConfigSaving ? (
              <LoaderCircle className="animate-spin" />
            ) : (
              t("notificationEmail.save")
            )}
          </Button>
        </CardFooter>
      </Card>
    </div>
  )
}
