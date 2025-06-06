"use client"

import { useTranslations } from "next-intl"
import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"

import { Input } from "@lana/web/ui/input"
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"

import { useCreateTermsTemplateMutation } from "@/lib/graphql/generated"
import { DEFAULT_TERMS } from "@/lib/constants/terms"
import { useModalNavigation } from "@/hooks/use-modal-navigation"

gql`
  mutation CreateTermsTemplate($input: TermsTemplateCreateInput!) {
    termsTemplateCreate(input: $input) {
      termsTemplate {
        ...TermsTemplateFields
      }
    }
  }
`

type CreateTermsTemplateDialogProps = {
  setOpenCreateTermsTemplateDialog: (isOpen: boolean) => void
  openCreateTermsTemplateDialog: boolean
}

export const CreateTermsTemplateDialog: React.FC<CreateTermsTemplateDialogProps> = ({
  setOpenCreateTermsTemplateDialog,
  openCreateTermsTemplateDialog,
}) => {
  const t = useTranslations("TermsTemplates.TermsTemplateDetails.CreateTermsTemplate")
  const { navigate, isNavigating } = useModalNavigation({
    closeModal: () => {
      setOpenCreateTermsTemplateDialog(false)
      resetForm()
    },
  })

  const [createTermsTemplate, { loading, error: createTermsTemplateError }] =
    useCreateTermsTemplateMutation({
      update: (cache) => {
        cache.modify({
          fields: {
            termsTemplates: (_, { DELETE }) => DELETE,
          },
        })
        cache.gc()
      },
    })

  const isLoading = loading || isNavigating

  const [formValues, setFormValues] = useState({
    name: "",
    annualRate: "",
    liquidationCvl: "",
    marginCallCvl: "",
    initialCvl: "",
    durationUnits: "",
    oneTimeFeeRate: "",
  })

  const [error, setError] = useState<string | null>(null)

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target
    setFormValues((prevValues) => ({
      ...prevValues,
      [name]: value,
    }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      await createTermsTemplate({
        variables: {
          input: {
            name: formValues.name,
            annualRate: formValues.annualRate,
            accrualCycleInterval: DEFAULT_TERMS.ACCRUAL_CYCLE_INTERVAL,
            accrualInterval: DEFAULT_TERMS.ACCRUAL_INTERVAL,
            duration: {
              period: DEFAULT_TERMS.DURATION_PERIOD,
              units: parseInt(formValues.durationUnits),
            },
            interestDueFromAccrualAfterDays: {
              period: DEFAULT_TERMS.INTEREST_DUE_FROM_ACCRUAL_AFTER_DAYS.PERIOD,
              units: DEFAULT_TERMS.INTEREST_DUE_FROM_ACCRUAL_AFTER_DAYS.UNITS,
            },
            obligationOverdueFromDueAfterDays: {
              period: DEFAULT_TERMS.OBLIGATION_OVERDUE_FROM_DUE_AFTER_DAYS.PERIOD,
              units: DEFAULT_TERMS.OBLIGATION_OVERDUE_FROM_DUE_AFTER_DAYS.UNITS,
            },
            liquidationCvl: formValues.liquidationCvl,
            marginCallCvl: formValues.marginCallCvl,
            initialCvl: formValues.initialCvl,
            oneTimeFeeRate: formValues.oneTimeFeeRate,
          },
        },
        onCompleted: (data) => {
          toast.success(t("success.created"))
          navigate(`/terms-templates/${data.termsTemplateCreate.termsTemplate.termsId}`)
        },
      })
    } catch (error) {
      console.error("Error creating Terms Template:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (createTermsTemplateError?.message) {
        setError(createTermsTemplateError.message)
      } else {
        setError(t("errors.general"))
      }
      toast.error(t("errors.creationFailed"))
    }
  }

  const resetForm = () => {
    setFormValues({
      name: "",
      annualRate: "",
      liquidationCvl: "",
      marginCallCvl: "",
      initialCvl: "",
      durationUnits: "",
      oneTimeFeeRate: "",
    })
    setError(null)
  }

  return (
    <Dialog
      open={openCreateTermsTemplateDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateTermsTemplateDialog(isOpen)
        if (!isOpen) {
          resetForm()
        }
      }}
    >
      <DialogContent className="max-w-[38rem]">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="name">{t("fields.name")}</Label>
            <Input
              id="name"
              name="name"
              type="text"
              required
              placeholder={t("placeholders.name")}
              value={formValues.name}
              onChange={handleChange}
              disabled={isLoading}
              data-testid="terms-template-name-input"
            />
          </div>
          <div className="grid auto-rows-fr sm:grid-cols-2 gap-4">
            <div className="space-y-4">
              <div>
                <Label htmlFor="annualRate">{t("fields.annualRate")}</Label>
                <Input
                  id="annualRate"
                  name="annualRate"
                  type="number"
                  required
                  placeholder={t("placeholders.annualRate")}
                  value={formValues.annualRate}
                  onChange={handleChange}
                  disabled={isLoading}
                  data-testid="terms-template-annual-rate-input"
                />
              </div>
              <div>
                <Label>{t("fields.duration")}</Label>
                <div className="flex gap-2 items-center">
                  <Input
                    type="number"
                    name="durationUnits"
                    value={formValues.durationUnits}
                    onChange={handleChange}
                    placeholder={t("placeholders.durationUnits")}
                    min={0}
                    required
                    disabled={isLoading}
                    data-testid="terms-template-duration-units-input"
                  />
                  <div className="p-1.5 bg-input-text rounded-md px-4">
                    {t("fields.months")}
                  </div>
                </div>
              </div>
              <div>
                <Label htmlFor="oneTimeFeeRate">{t("fields.oneTimeFeeRate")}</Label>
                <Input
                  id="oneTimeFeeRate"
                  name="oneTimeFeeRate"
                  type="number"
                  required
                  placeholder={t("placeholders.oneTimeFeeRate")}
                  value={formValues.oneTimeFeeRate}
                  onChange={handleChange}
                  disabled={isLoading}
                  data-testid="terms-template-one-time-fee-rate-input"
                />
              </div>
            </div>
            <div className="space-y-4">
              <div>
                <Label htmlFor="initialCvl">{t("fields.initialCvl")}</Label>
                <Input
                  id="initialCvl"
                  name="initialCvl"
                  type="number"
                  required
                  placeholder={t("placeholders.initialCvl")}
                  value={formValues.initialCvl}
                  onChange={handleChange}
                  disabled={isLoading}
                  data-testid="terms-template-initial-cvl-input"
                />
              </div>
              <div>
                <Label htmlFor="marginCallCvl">{t("fields.marginCallCvl")}</Label>
                <Input
                  id="marginCallCvl"
                  name="marginCallCvl"
                  type="number"
                  required
                  placeholder={t("placeholders.marginCallCvl")}
                  value={formValues.marginCallCvl}
                  onChange={handleChange}
                  disabled={isLoading}
                  data-testid="terms-template-margin-call-cvl-input"
                />
              </div>
              <div>
                <Label htmlFor="liquidationCvl">{t("fields.liquidationCvl")}</Label>
                <Input
                  id="liquidationCvl"
                  name="liquidationCvl"
                  type="number"
                  required
                  placeholder={t("placeholders.liquidationCvl")}
                  value={formValues.liquidationCvl}
                  onChange={handleChange}
                  disabled={isLoading}
                  data-testid="terms-template-liquidation-cvl-input"
                />
              </div>
            </div>
          </div>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="submit"
              loading={isLoading}
              data-testid="terms-template-submit-button"
            >
              {t("buttons.submit")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
