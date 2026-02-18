"use client"

import { Button } from "@lana/web/ui/button"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Label } from "@lana/web/ui/label"
import { cn } from "@lana/web/utils"
import { useTranslations } from "next-intl"
import { FormEvent, useEffect, useMemo, useState } from "react"

import {
  AccountSetCombobox,
  formatOptionValue,
  type AccountSetOptionBase,
} from "@/app/components/account-set-combobox"

type FormDataShape = Record<string, string>

export type ModuleConfigField<
  TFormData extends FormDataShape,
  TCategory extends string,
  TGroup extends string,
> = {
  key: Extract<keyof TFormData, string>
  category: TCategory
  group: TGroup
}

type ModuleFieldGroup<TGroup extends string> = {
  key: TGroup
  titleKey: string
}

type ModuleFieldChange<TField> = {
  field: TField
  from: string
  to: string
}

type AccountSetOption<TCategory extends string> = AccountSetOptionBase & {
  category: TCategory
}

type ModuleConfigUpdateDialogProps<
  TFormData extends FormDataShape,
  TCategory extends string,
  TGroup extends string,
  TField extends ModuleConfigField<TFormData, TCategory, TGroup>,
  TModuleConfig,
> = {
  open: boolean
  setOpen: (isOpen: boolean) => void
  moduleKey: string
  moduleConfig?: TModuleConfig
  accountSetOptions?: AccountSetOption<TCategory>[]
  accountSetOptionsError?: boolean
  fields: TField[]
  fieldGroups: Array<ModuleFieldGroup<TGroup>>
  emptyFormData: TFormData
  buildFormDataFromConfig: (moduleConfig?: TModuleConfig) => TFormData
  buildChanges: (
    baseline: TFormData,
    current: TFormData,
  ) => Array<ModuleFieldChange<TField>>
  loading: boolean
  errorMessage?: string
  reset: () => void
  onSave: (input: TFormData) => Promise<void>
}

export const ModuleConfigUpdateDialog = <
  TFormData extends FormDataShape,
  TCategory extends string,
  TGroup extends string,
  TField extends ModuleConfigField<TFormData, TCategory, TGroup>,
  TModuleConfig,
>({
  open,
  setOpen,
  moduleKey,
  moduleConfig,
  accountSetOptions = [],
  accountSetOptionsError = false,
  fields,
  fieldGroups,
  emptyFormData,
  buildFormDataFromConfig,
  buildChanges,
  loading,
  errorMessage,
  reset,
  onSave,
}: ModuleConfigUpdateDialogProps<
  TFormData,
  TCategory,
  TGroup,
  TField,
  TModuleConfig
>) => {
  const t = useTranslations("Modules")
  const tCommon = useTranslations("Common")
  const tModule = (key: string) => t(`${moduleKey}.${key}` as never)
  const tAccountCategory = (category: TCategory) =>
    t(`accountCategories.${category}` as never)

  const [step, setStep] = useState<"edit" | "confirm">("edit")
  const [baselineFormData, setBaselineFormData] = useState<TFormData>({
    ...emptyFormData,
  })
  const [formData, setFormData] = useState<TFormData>({ ...emptyFormData })
  const accountSetOptionsByCategory = useMemo(() => {
    const grouped: Partial<Record<TCategory, AccountSetOption<TCategory>[]>> = {}

    accountSetOptions.forEach((option) => {
      const categoryOptions = grouped[option.category] ?? []
      categoryOptions.push(option)
      grouped[option.category] = categoryOptions
    })

    return grouped
  }, [accountSetOptions])
  const changes = useMemo(
    () => buildChanges(baselineFormData, formData),
    [baselineFormData, buildChanges, formData],
  )
  const hasChanges = changes.length > 0

  const close = () => {
    reset()
    setOpen(false)
    setFormData({ ...emptyFormData })
    setBaselineFormData({ ...emptyFormData })
    setStep("edit")
  }

  useEffect(() => {
    if (!open) return
    const updatedFormData = buildFormDataFromConfig(moduleConfig)
    setBaselineFormData(updatedFormData)
    setFormData(updatedFormData)
    setStep("edit")
  }, [buildFormDataFromConfig, moduleConfig, open])

  const submit = (e: FormEvent) => {
    e.preventDefault()
    reset()
    setStep("confirm")
  }

  const handleDone = async () => {
    try {
      await onSave(formData)
      close()
    } catch {
      // error is rendered inline in confirmation view
    }
  }

  const handleBack = () => {
    reset()
    setStep("edit")
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) close()
      }}
    >
      <DialogContent className="max-h-[calc(100vh-2rem)] overflow-y-auto sm:max-w-4xl">
        <DialogHeader>
          <DialogTitle>
            {step === "confirm" ? tModule("confirmationTitle") : tModule("setTitle")}
          </DialogTitle>
        </DialogHeader>
        {step === "confirm" ? (
          <>
            <p className="text-sm text-muted-foreground">
              {tModule("confirmationDescription")}
            </p>
            {changes.length > 0 && (
              <div className="mt-4 space-y-3">
                {changes.map(({ field, from, to }) => {
                  const optionsForCategory =
                    accountSetOptionsByCategory[field.category] ?? []
                  const previousLabel = formatOptionValue(from, optionsForCategory)
                  const updatedLabel = formatOptionValue(to, optionsForCategory)
                  const previousEmpty = !from
                  const updatedEmpty = !to

                  return (
                    <div
                      key={field.key}
                      className="space-y-2 rounded-lg border border-border p-3"
                    >
                      <div className="text-sm font-medium">
                        {tModule(field.key)}
                      </div>
                      <div className="grid gap-3 sm:grid-cols-2">
                        <div className="space-y-1">
                          <div className="text-xs text-muted-foreground">
                            {tModule("confirmationPrevious")}
                          </div>
                          <div
                            className={cn(
                              "min-h-[2rem] rounded-md border px-2 py-1 text-sm",
                              previousEmpty
                                ? "border-amber-400/70 text-amber-700"
                                : "border-border",
                            )}
                          >
                            {previousEmpty ? "\u00A0" : previousLabel}
                          </div>
                        </div>
                        <div className="space-y-1">
                          <div className="text-xs text-muted-foreground">
                            {tModule("confirmationUpdated")}
                          </div>
                          <div
                            className={cn(
                              "min-h-[2rem] rounded-md border px-2 py-1 text-sm",
                              updatedEmpty
                                ? "border-amber-400/70 text-amber-700"
                                : "border-border",
                            )}
                          >
                            {updatedEmpty ? "\u00A0" : updatedLabel}
                          </div>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
            {errorMessage && <div className="text-destructive">{errorMessage}</div>}
            <DialogFooter className="mt-4">
              <Button variant="outline" type="button" onClick={handleBack} disabled={loading}>
                {tCommon("back")}
              </Button>
              <Button loading={loading} type="button" onClick={handleDone}>
                {tCommon("save")}
              </Button>
            </DialogFooter>
          </>
        ) : (
          <form onSubmit={submit}>
            <div className="flex flex-col space-y-6 w-full">
              {accountSetOptionsError && (
                <div className="text-sm text-destructive">{tCommon("error")}</div>
              )}
              {fieldGroups.map((group) => {
                const fieldsForGroup = fields.filter((field) => field.group === group.key)

                return (
                  <div
                    key={group.key}
                    className="space-y-3 rounded-lg border border-border bg-muted/30 p-4"
                  >
                    <div className="text-sm font-semibold">
                      {tModule(`groups.${group.titleKey}`)}
                    </div>
                    <div className="flex flex-col space-y-2 w-full">
                      {fieldsForGroup.map((field) => {
                        const optionsForCategory =
                          accountSetOptionsByCategory[field.category] ?? []
                        const isDisabled = optionsForCategory.length === 0
                        const handleChange = (nextValue: string) => {
                          setFormData((previous) => ({
                            ...previous,
                            [field.key]: nextValue,
                          }))
                        }

                        return (
                          <div key={field.key}>
                            <div className="flex items-center justify-between gap-2">
                              <Label htmlFor={field.key}>
                                {tModule(field.key)}
                              </Label>
                              <span className="text-xs text-muted-foreground">
                                {tAccountCategory(field.category)}
                              </span>
                            </div>
                            <AccountSetCombobox
                              id={field.key}
                              value={formData[field.key]}
                              options={optionsForCategory}
                              onChange={handleChange}
                              disabled={isDisabled}
                              placeholder={tModule("accountSetSelectPlaceholder")}
                              searchPlaceholder={tModule("accountSetSearchPlaceholder")}
                              emptyLabel={tModule("accountSetEmpty")}
                            />
                          </div>
                        )
                      })}
                    </div>
                  </div>
                )
              })}
            </div>
            {errorMessage && <div className="text-destructive">{errorMessage}</div>}
            <DialogFooter className="mt-4">
              <Button variant="outline" type="button" onClick={close}>
                {tCommon("cancel")}
              </Button>
              <Button loading={loading} type="submit" disabled={!hasChanges || loading}>
                {tCommon("review")}
              </Button>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  )
}
