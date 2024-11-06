import React, { useState, useEffect } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import {
  useUpdateTermsTemplateMutation,
  InterestInterval,
  Period,
  TermsTemplate,
} from "@/lib/graphql/generated"
import { Input } from "@/components/primitive/input"
import { Button } from "@/components/primitive/button"
import { Label } from "@/components/primitive/label"
import { Select } from "@/components/primitive/select"
import { formatInterval, formatPeriod } from "@/lib/utils"

gql`
  mutation UpdateTermsTemplate($input: TermsTemplateUpdateInput!) {
    termsTemplateUpdate(input: $input) {
      termsTemplate {
        id
        termsId
        name
        values {
          annualRate
          accrualInterval
          incurrenceInterval
          liquidationCvl
          marginCallCvl
          initialCvl
          duration {
            period
            units
          }
        }
      }
    }
  }
`

type UpdateTermsTemplateDialogProps = {
  setOpenUpdateTermsTemplateDialog: (isOpen: boolean) => void
  openUpdateTermsTemplateDialog: boolean
  refetch?: () => void
  termsTemplate: TermsTemplate
}

export const UpdateTermsTemplateDialog: React.FC<UpdateTermsTemplateDialogProps> = ({
  setOpenUpdateTermsTemplateDialog,
  openUpdateTermsTemplateDialog,
  refetch,
  termsTemplate,
}) => {
  const [updateTermsTemplate, { loading, reset, error: updateTermsTemplateError }] =
    useUpdateTermsTemplateMutation()

  const [formValues, setFormValues] = useState({
    name: termsTemplate.name,
    annualRate: termsTemplate.values.annualRate.toString(),
    accrualInterval: termsTemplate.values.accrualInterval,
    incurrenceInterval: termsTemplate.values.incurrenceInterval,
    durationUnits: termsTemplate.values.duration.units.toString(),
    durationPeriod: termsTemplate.values.duration.period,
    liquidationCvl: termsTemplate.values.liquidationCvl.toString(),
    marginCallCvl: termsTemplate.values.marginCallCvl.toString(),
    initialCvl: termsTemplate.values.initialCvl.toString(),
  })

  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (openUpdateTermsTemplateDialog) {
      setFormValues({
        name: termsTemplate.name,
        annualRate: termsTemplate.values.annualRate.toString(),
        accrualInterval: termsTemplate.values.accrualInterval,
        incurrenceInterval: termsTemplate.values.incurrenceInterval,
        durationUnits: termsTemplate.values.duration.units.toString(),
        durationPeriod: termsTemplate.values.duration.period,
        liquidationCvl: termsTemplate.values.liquidationCvl.toString(),
        marginCallCvl: termsTemplate.values.marginCallCvl.toString(),
        initialCvl: termsTemplate.values.initialCvl.toString(),
      })
    }
  }, [openUpdateTermsTemplateDialog, termsTemplate])

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
      const { data } = await updateTermsTemplate({
        variables: {
          input: {
            id: termsTemplate.termsId,
            annualRate: formValues.annualRate,
            accrualInterval: formValues.accrualInterval as InterestInterval,
            incurrenceInterval: formValues.incurrenceInterval as InterestInterval,
            duration: {
              period: formValues.durationPeriod as Period,
              units: parseInt(formValues.durationUnits),
            },
            liquidationCvl: formValues.liquidationCvl,
            marginCallCvl: formValues.marginCallCvl,
            initialCvl: formValues.initialCvl,
          },
        },
      })
      if (data?.termsTemplateUpdate.termsTemplate) {
        toast.success("Terms Template updated successfully")
        if (refetch) refetch()
        setOpenUpdateTermsTemplateDialog(false)
      } else {
        throw new Error("Failed to update Terms Template. Please try again.")
      }
    } catch (error) {
      console.error("Error updating Terms Template:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (updateTermsTemplateError?.message) {
        setError(updateTermsTemplateError.message)
      } else {
        setError("An unexpected error occurred. Please try again.")
      }
      toast.error("Failed to update Terms Template")
    }
  }

  const resetForm = () => {
    setFormValues({
      name: termsTemplate.name,
      annualRate: termsTemplate.values.annualRate.toString(),
      accrualInterval: termsTemplate.values.accrualInterval,
      incurrenceInterval: termsTemplate.values.incurrenceInterval,
      durationUnits: termsTemplate.values.duration.units.toString(),
      durationPeriod: termsTemplate.values.duration.period,
      liquidationCvl: termsTemplate.values.liquidationCvl.toString(),
      marginCallCvl: termsTemplate.values.marginCallCvl.toString(),
      initialCvl: termsTemplate.values.initialCvl.toString(),
    })
    setError(null)
    reset()
  }

  return (
    <Dialog
      open={openUpdateTermsTemplateDialog}
      onOpenChange={(isOpen) => {
        setOpenUpdateTermsTemplateDialog(isOpen)
        if (!isOpen) {
          resetForm()
        }
      }}
    >
      <DialogContent className="max-w-[38rem]">
        <DialogHeader>
          <DialogTitle>Update {formValues.name}</DialogTitle>
          <DialogDescription>
            Update the Terms Template by modifying the required information
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-4">
              <div>
                <Label htmlFor="annualRate">Interest Rate (APR)</Label>
                <Input
                  id="annualRate"
                  name="annualRate"
                  type="number"
                  required
                  placeholder="Enter the annual rate"
                  value={formValues.annualRate}
                  onChange={handleChange}
                />
              </div>
              <div>
                <Label>Duration</Label>
                <div className="flex gap-2">
                  <Input
                    type="number"
                    name="durationUnits"
                    value={formValues.durationUnits}
                    onChange={handleChange}
                    placeholder="Duration"
                    min={0}
                    required
                    className="w-1/2"
                  />
                  <Select
                    name="durationPeriod"
                    value={formValues.durationPeriod}
                    onChange={handleChange}
                    required
                  >
                    {Object.values(Period).map((period) => (
                      <option key={period} value={period}>
                        {formatPeriod(period)}
                      </option>
                    ))}
                  </Select>
                </div>
              </div>
              <div>
                <Label htmlFor="accrualInterval">Accrual Interval</Label>
                <Select
                  id="accrualInterval"
                  name="accrualInterval"
                  value={formValues.accrualInterval}
                  onChange={handleChange}
                  required
                >
                  {Object.values(InterestInterval).map((int) => (
                    <option key={int} value={int}>
                      {formatInterval(int)}
                    </option>
                  ))}
                </Select>
              </div>
              <div>
                <Label htmlFor="incurrenceInterval">Incurrence Interval</Label>
                <Select
                  id="incurrenceInterval"
                  name="incurrenceInterval"
                  value={formValues.incurrenceInterval}
                  onChange={handleChange}
                  required
                >
                  {Object.values(InterestInterval).map((int) => (
                    <option key={int} value={int}>
                      {formatInterval(int)}
                    </option>
                  ))}
                </Select>
              </div>
            </div>
            <div className="space-y-4">
              <div>
                <Label htmlFor="initialCvl">Initial CVL (%)</Label>
                <Input
                  id="initialCvl"
                  name="initialCvl"
                  type="number"
                  required
                  placeholder="Enter the initial CVL"
                  value={formValues.initialCvl}
                  onChange={handleChange}
                />
              </div>
              <div>
                <Label htmlFor="marginCallCvl">Margin Call CVL (%)</Label>
                <Input
                  id="marginCallCvl"
                  name="marginCallCvl"
                  type="number"
                  required
                  placeholder="Enter the margin call CVL"
                  value={formValues.marginCallCvl}
                  onChange={handleChange}
                />
              </div>
              <div>
                <Label htmlFor="liquidationCvl">Liquidation CVL (%)</Label>
                <Input
                  id="liquidationCvl"
                  name="liquidationCvl"
                  type="number"
                  required
                  placeholder="Enter the liquidation CVL"
                  value={formValues.liquidationCvl}
                  onChange={handleChange}
                />
              </div>
            </div>
          </div>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button type="submit" loading={loading}>
              Update Terms Template
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
