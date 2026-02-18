"use client"

import { gql } from "@apollo/client"
import { useEffect, useMemo, useState } from "react"
import { useTranslations } from "next-intl"
import { CheckIcon, ChevronsUpDownIcon, LoaderCircle } from "lucide-react"
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
import { Checkbox } from "@lana/web/ui/checkbox"
import { Popover, PopoverContent, PopoverTrigger } from "@lana/web/ui/popover"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@lana/web/ui/command"
import { cn } from "@lana/web/utils"

import {
  ConfigType,
  type DomainConfig,
  useDomainConfigsQuery,
  useDomainConfigUpdateMutation,
} from "@/lib/graphql/generated"

gql`
  query DomainConfigs($first: Int!, $after: String) {
    domainConfigs(first: $first, after: $after) {
      nodes {
        id
        domainConfigId
        key
        configType
        encrypted
        isSet
        value
      }
      pageInfo {
        hasNextPage
        endCursor
      }
    }
  }

  mutation DomainConfigUpdate($input: DomainConfigUpdateInput!) {
    domainConfigUpdate(input: $input) {
      domainConfig {
        id
        domainConfigId
        key
        configType
        encrypted
        isSet
        value
      }
    }
  }
`

const DOMAIN_CONFIG_PAGE_SIZE = 100
const EMPTY_CONFIGS: DomainConfig[] = []

// Get all IANA timezones from the browser's Intl API
const ALL_TIMEZONES = Intl.supportedValuesOf("timeZone")

export default function ConfigurationsPage() {
  const t = useTranslations("Configurations")

  const [domainDrafts, setDomainDrafts] = useState<
    Record<string, string | boolean>
  >({})

  const {
    data: domainConfigData,
    loading: domainConfigLoading,
    error: domainConfigError,
  } = useDomainConfigsQuery({
    variables: {
      first: DOMAIN_CONFIG_PAGE_SIZE,
    },
  })

  const [domainConfigUpdate, { loading: domainConfigUpdateLoading }] =
    useDomainConfigUpdateMutation()

  const domainConfigs = domainConfigData?.domainConfigs.nodes ?? EMPTY_CONFIGS
  const visibleConfigs = useMemo(
    () => domainConfigs.filter((config) => config.configType !== ConfigType.Complex),
    [domainConfigs],
  )

  useEffect(() => {
    if (visibleConfigs.length === 0) {
      setDomainDrafts({})
      return
    }

    setDomainDrafts((prev) => {
      const nextDrafts: Record<string, string | boolean> = {}

      for (const config of visibleConfigs) {
        if (prev[config.key] !== undefined) {
          nextDrafts[config.key] = prev[config.key]
        } else {
          nextDrafts[config.key] = formatDomainValue(config)
        }
      }

      return nextDrafts
    })
  }, [visibleConfigs])

  const handleDomainSave = async (config: DomainConfig) => {
    const draft = domainDrafts[config.key]
    const parsed = parseDomainDraft(config, draft)

    if ("errorKey" in parsed) {
      toast.error(t(parsed.errorKey))
      return
    }

    try {
      const result = await domainConfigUpdate({
        variables: {
          input: {
            domainConfigId: config.domainConfigId,
            value: parsed.value,
          },
        },
      })

      const updated = result.data?.domainConfigUpdate.domainConfig

      if (!updated) {
        toast.error(t("domainConfigs.saveError"))
        return
      }

      toast.success(t("domainConfigs.saveSuccess"))
      setDomainDrafts((prev) => ({
        ...prev,
        [config.key]: formatDomainValue(updated),
      }))
    } catch (error) {
      console.error("Failed to update domain configuration:", error)

      const errorMessage = error instanceof Error ? error.message : null

      toast.error(
        errorMessage
          ? t("domainConfigs.saveErrorWithReason", { error: errorMessage })
          : t("domainConfigs.saveError"),
      )
    }
  }

  return (
    <div className="space-y-3">
      {domainConfigLoading ? (
        <LoaderCircle className="animate-spin" />
      ) : domainConfigError ? (
        <p className="text-sm text-destructive">
          {t("domainConfigs.loadError")}
        </p>
      ) : visibleConfigs.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {t("domainConfigs.empty")}
        </p>
      ) : (
        <div className="space-y-3">
          {visibleConfigs.map((config) => {
            const inputId = `domain-${config.key}`
            const isDisabled = domainConfigLoading

            return (
              <Card key={config.key}>
                <CardHeader>
                  <CardTitle>{t(`${config.key}.title`)}</CardTitle>
                  <CardDescription>
                    {t(`${config.key}.description`)}
                  </CardDescription>
                </CardHeader>
                <CardContent className="grid gap-4">
                  {renderDomainInput({
                    config,
                    inputId,
                    value: domainDrafts[config.key],
                    disabled: isDisabled,
                    onChange: (nextValue) =>
                      setDomainDrafts((prev) => ({
                        ...prev,
                        [config.key]: nextValue,
                      })),
                  })}
                </CardContent>
                <CardFooter className="justify-end">
                  <Button
                    onClick={() => handleDomainSave(config)}
                    disabled={isDisabled}
                    loading={domainConfigUpdateLoading}
                  >
                    {t("domainConfigs.save")}
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

type TimezoneComboboxProps = {
  value: string
  onChange: (value: string) => void
  disabled?: boolean
  id?: string
}

function TimezoneCombobox({ value, onChange, disabled, id }: TimezoneComboboxProps) {
  const [open, setOpen] = useState(false)

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          id={id}
          variant="outline"
          role="combobox"
          aria-expanded={open}
          disabled={disabled}
          className="w-full justify-between font-normal"
        >
          {value || "Select timezone..."}
          <ChevronsUpDownIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[--radix-popover-trigger-width] p-0" align="start">
        <Command>
          <CommandInput placeholder="Search timezone..." />
          <CommandList>
            <CommandEmpty>No timezone found.</CommandEmpty>
            <CommandGroup>
              {ALL_TIMEZONES.map((tz) => (
                <CommandItem
                  key={tz}
                  value={tz}
                  onSelect={(selectedValue) => {
                    onChange(selectedValue)
                    setOpen(false)
                  }}
                >
                  <CheckIcon
                    className={cn(
                      "mr-2 h-4 w-4",
                      value === tz ? "opacity-100" : "opacity-0",
                    )}
                  />
                  {tz}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}

const formatDomainValue = (config: DomainConfig): string | boolean => {
  if (config.encrypted) {
    return ""
  }

  switch (config.configType) {
    case ConfigType.Bool:
      return config.value === true
    case ConfigType.String:
    case ConfigType.Timezone:
    case ConfigType.Time:
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

const parseDomainDraft = (
  config: DomainConfig,
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
    case ConfigType.Timezone: {
      const text = typeof draft === "string" ? draft.trim() : ""

      if (text.length === 0) {
        return { errorKey: "domainConfigs.invalidTimezone" }
      }

      return { value: text }
    }
    case ConfigType.Time: {
      const text = typeof draft === "string" ? draft.trim() : ""

      if (text.length === 0) {
        return { errorKey: "domainConfigs.invalidTime" }
      }

      // Accept HH:MM or HH:MM:SS, normalize to HH:MM:SS
      if (/^\d{2}:\d{2}$/.test(text)) {
        return { value: `${text}:00` }
      }

      if (/^\d{2}:\d{2}:\d{2}$/.test(text)) {
        return { value: text }
      }

      return { errorKey: "domainConfigs.invalidTime" }
    }
    case ConfigType.Int: {
      const text = typeof draft === "string" ? draft.trim() : ""
      const parsed = Number(text)

      if (text.length === 0) {
        return { errorKey: "domainConfigs.invalidInt" }
      }

      if (!Number.isInteger(parsed)) {
        return { errorKey: "domainConfigs.invalidInt" }
      }

      return { value: parsed }
    }
    case ConfigType.Uint: {
      const text = typeof draft === "string" ? draft.trim() : ""
      const parsed = Number(text)

      if (text.length === 0) {
        return { errorKey: "domainConfigs.invalidUint" }
      }

      if (!Number.isInteger(parsed) || parsed < 0) {
        return { errorKey: "domainConfigs.invalidUint" }
      }

      return { value: parsed }
    }
    case ConfigType.Decimal: {
      const text = typeof draft === "string" ? draft.trim() : ""

      if (text.length === 0) {
        return { errorKey: "domainConfigs.invalidDecimal" }
      }

      return { value: text }
    }
    default:
      return { errorKey: "domainConfigs.invalidValue" }
  }
}

type RenderDomainInputArgs = {
  config: DomainConfig
  inputId: string
  value: string | boolean | undefined
  disabled: boolean
  onChange: (value: string | boolean) => void
}

const renderDomainInput = ({
  config,
  inputId,
  value,
  disabled,
  onChange,
}: RenderDomainInputArgs) => {
  if (config.encrypted) {
    return (
      <Input
        id={inputId}
        type="password"
        placeholder={config.isSet ? "••••••" : ""}
        value={typeof value === "string" ? value : ""}
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
      />
    )
  }

  switch (config.configType) {
    case ConfigType.Bool:
      return (
        <Checkbox
          id={inputId}
          checked={value === true}
          disabled={disabled}
          onCheckedChange={(checked) => onChange(checked === true)}
        />
      )
    case ConfigType.Int:
    case ConfigType.Uint:
      return (
        <Input
          id={inputId}
          type="number"
          value={typeof value === "string" ? value : ""}
          disabled={disabled}
          onChange={(event) => onChange(event.target.value)}
        />
      )
    case ConfigType.Decimal:
      return (
        <Input
          id={inputId}
          inputMode="decimal"
          value={typeof value === "string" ? value : ""}
          disabled={disabled}
          onChange={(event) => onChange(event.target.value)}
        />
      )
    case ConfigType.Timezone:
      return (
        <TimezoneCombobox
          id={inputId}
          value={typeof value === "string" ? value : ""}
          onChange={onChange}
          disabled={disabled}
        />
      )
    case ConfigType.Time:
      return (
        <Input
          id={inputId}
          type="time"
          step="1"
          value={typeof value === "string" ? value : ""}
          disabled={disabled}
          onChange={(event) => onChange(event.target.value)}
        />
      )
    case ConfigType.String:
    default:
      return (
        <Input
          id={inputId}
          value={typeof value === "string" ? value : ""}
          disabled={disabled}
          onChange={(event) => onChange(event.target.value)}
        />
      )
  }
}
