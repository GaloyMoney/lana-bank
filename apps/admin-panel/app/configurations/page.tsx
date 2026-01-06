"use client"

import { gql } from "@apollo/client"
import { useEffect, useState } from "react"
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
import { Checkbox } from "@lana/web/ui/checkbox"

import {
  ConfigType,
  type ExposedConfigItem,
  ExposedConfigsDocument,
  useExposedConfigsQuery,
  useUpdateExposedConfigMutation,
} from "@/lib/graphql/generated"

gql`
  query ExposedConfigs {
    exposedConfigs {
      key
      configType
      value
      isSet
    }
  }

  mutation UpdateExposedConfig($input: ExposedConfigUpdateInput!) {
    updateExposedConfig(input: $input) {
      exposedConfig {
        key
        configType
        value
        isSet
      }
    }
  }
`

const EMPTY_CONFIGS: ExposedConfigItem[] = []

export default function ConfigurationsPage() {
  const t = useTranslations("Configurations")

  const [exposedDrafts, setExposedDrafts] = useState<
    Record<string, string | boolean>
  >({})

  const {
    data: exposedConfigData,
    loading: exposedConfigLoading,
    error: exposedConfigError,
  } = useExposedConfigsQuery()

  const [updateExposedConfig, { loading: updateExposedConfigLoading }] =
    useUpdateExposedConfigMutation()

  const exposedConfigs = exposedConfigData?.exposedConfigs ?? EMPTY_CONFIGS
  const visibleConfigs = exposedConfigs.filter(
    (config) => config.configType !== ConfigType.Complex,
  )

  useEffect(() => {
    const nextVisibleConfigs = exposedConfigs.filter(
      (config) => config.configType !== ConfigType.Complex,
    )

    if (nextVisibleConfigs.length === 0) {
      setExposedDrafts({})
      return
    }

    setExposedDrafts((prev) => {
      const nextDrafts: Record<string, string | boolean> = {}

      for (const config of nextVisibleConfigs) {
        if (prev[config.key] !== undefined) {
          nextDrafts[config.key] = prev[config.key]
        } else {
          nextDrafts[config.key] = formatExposedValue(config)
        }
      }

      return nextDrafts
    })
  }, [exposedConfigs])

  const handleExposedSave = async (config: ExposedConfigItem) => {
    const draft = exposedDrafts[config.key]
    const parsed = parseExposedDraft(config, draft)

    if ("errorKey" in parsed) {
      toast.error(t(parsed.errorKey))
      return
    }

    try {
      const result = await updateExposedConfig({
        variables: {
          input: {
            key: config.key,
            value: parsed.value,
          },
        },
        refetchQueries: [ExposedConfigsDocument],
      })

      const updated = result.data?.updateExposedConfig.exposedConfig

      if (!updated) {
        toast.error(t("exposedConfigs.saveError"))
        return
      }

      toast.success(t("exposedConfigs.saveSuccess"))
      setExposedDrafts((prev) => ({
        ...prev,
        [config.key]: formatExposedValue(updated),
      }))
    } catch (error) {
      console.error("Failed to update exposed configuration:", error)

      const errorMessage = error instanceof Error ? error.message : null

      toast.error(
        errorMessage
          ? t("exposedConfigs.saveErrorWithReason", { error: errorMessage })
          : t("exposedConfigs.saveError"),
      )
    }
  }

  return (
    <div className="space-y-3">
      {exposedConfigLoading ? (
        <LoaderCircle className="animate-spin" />
      ) : exposedConfigError ? (
        <p className="text-sm text-destructive">
          {t("exposedConfigs.loadError")}
        </p>
      ) : visibleConfigs.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {t("exposedConfigs.empty")}
        </p>
      ) : (
        <div className="space-y-3">
          {visibleConfigs.map((config) => {
            const inputId = `exposed-${config.key}`
            const isDisabled = exposedConfigLoading

            return (
              <Card key={config.key}>
                <CardHeader>
                  <CardTitle>{t(`${config.key}.title`)}</CardTitle>
                  <CardDescription>
                    {t(`${config.key}.description`)}
                  </CardDescription>
                </CardHeader>
                <CardContent className="grid gap-4">
                  {renderExposedInput({
                    config,
                    inputId,
                    value: exposedDrafts[config.key],
                    disabled: isDisabled,
                    onChange: (nextValue) =>
                      setExposedDrafts((prev) => ({
                        ...prev,
                        [config.key]: nextValue,
                      })),
                    label: t("exposedConfigs.valueLabel"),
                  })}
                </CardContent>
                <CardFooter className="justify-end">
                  <Button
                    onClick={() => handleExposedSave(config)}
                    disabled={isDisabled}
                    loading={updateExposedConfigLoading}
                  >
                    {t("exposedConfigs.save")}
                  </Button>
                </CardFooter>
              </Card>
            )
          })}
        </div>
      )}
    </div>
  )
}

const formatExposedValue = (config: ExposedConfigItem): string | boolean => {
  switch (config.configType) {
    case ConfigType.Bool:
      return config.value === true
    case ConfigType.String:
      return typeof config.value === "string" ? config.value : ""
    case ConfigType.Int:
    case ConfigType.Uint:
      if (typeof config.value === "number") {
        return config.value.toString()
      }
      return typeof config.value === "string" ? config.value : ""
    case ConfigType.Decimal:
      return typeof config.value === "string" ? config.value : ""
    default:
      return ""
  }
}

const parseExposedDraft = (
  config: ExposedConfigItem,
  draft: string | boolean | undefined,
):
  | { value: unknown }
  | {
      errorKey: string
    } => {
  switch (config.configType) {
    case ConfigType.Bool:
      return { value: draft === true }
    case ConfigType.String:
      return { value: typeof draft === "string" ? draft : "" }
    case ConfigType.Int: {
      const text = typeof draft === "string" ? draft.trim() : ""
      const parsed = Number(text)

      if (text.length === 0) {
        return { errorKey: "exposedConfigs.invalidInt" }
      }

      if (!Number.isInteger(parsed)) {
        return { errorKey: "exposedConfigs.invalidInt" }
      }

      return { value: parsed }
    }
    case ConfigType.Uint: {
      const text = typeof draft === "string" ? draft.trim() : ""
      const parsed = Number(text)

      if (text.length === 0) {
        return { errorKey: "exposedConfigs.invalidUint" }
      }

      if (!Number.isInteger(parsed) || parsed < 0) {
        return { errorKey: "exposedConfigs.invalidUint" }
      }

      return { value: parsed }
    }
    case ConfigType.Decimal: {
      const text = typeof draft === "string" ? draft.trim() : ""

      if (text.length === 0) {
        return { errorKey: "exposedConfigs.invalidDecimal" }
      }

      return { value: text }
    }
    default:
      return { errorKey: "exposedConfigs.invalidValue" }
  }
}

type RenderExposedInputArgs = {
  config: ExposedConfigItem
  inputId: string
  value: string | boolean | undefined
  disabled: boolean
  onChange: (value: string | boolean) => void
  label: string
}

const renderExposedInput = ({
  config,
  inputId,
  value,
  disabled,
  onChange,
  label,
}: RenderExposedInputArgs) => {
  switch (config.configType) {
    case ConfigType.Bool:
      return (
        <div className="flex items-center gap-2">
          <Checkbox
            id={inputId}
            checked={value === true}
            disabled={disabled}
            onCheckedChange={(checked) => onChange(checked === true)}
          />
          <Label htmlFor={inputId}>{label}</Label>
        </div>
      )
    case ConfigType.Int:
    case ConfigType.Uint:
      return (
        <div className="grid gap-2">
          <Label htmlFor={inputId}>{label}</Label>
          <Input
            id={inputId}
            type="number"
            value={typeof value === "string" ? value : ""}
            disabled={disabled}
            onChange={(event) => onChange(event.target.value)}
          />
        </div>
      )
    case ConfigType.Decimal:
      return (
        <div className="grid gap-2">
          <Label htmlFor={inputId}>{label}</Label>
          <Input
            id={inputId}
            inputMode="decimal"
            value={typeof value === "string" ? value : ""}
            disabled={disabled}
            onChange={(event) => onChange(event.target.value)}
          />
        </div>
      )
    case ConfigType.String:
    default:
      return (
        <div className="grid gap-2">
          <Label htmlFor={inputId}>{label}</Label>
          <Input
            id={inputId}
            value={typeof value === "string" ? value : ""}
            disabled={disabled}
            onChange={(event) => onChange(event.target.value)}
          />
        </div>
      )
  }
}
