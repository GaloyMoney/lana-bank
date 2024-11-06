import React, { useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { useRouter } from "next/navigation"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import {
  useCreateTermsTemplateMutation,
  InterestInterval,
  Period,
} from "@/lib/graphql/generated"
import { Input } from "@/components/primitive/input"
import { Button } from "@/components/primitive/button"
import { Label } from "@/components/primitive/label"
import { Select } from "@/components/primitive/select"
import { formatInterval, formatPeriod } from "@/lib/utils"

gql`
  mutation CreateTermsTemplate($input: TermsTemplateCreateInput!) {
    termsTemplateCreate(input: $input) {
      termsTemplate {
        id
        termsId
        values {
          annualRate
          accrualInterval
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

type CreateTermsTemplateDialogProps = {
  setOpenCreateTermsTemplateDialog: (isOpen: boolean) => void
  openCreateTermsTemplateDialog: boolean
  refetch?: () => void
}

export const CreateTermsTemplateDialog: React.FC<CreateTermsTemplateDialogProps> = ({
  setOpenCreateTermsTemplateDialog,
  openCreateTermsTemplateDialog,
  refetch,
}) => {
  const router = useRouter()

  const [createTermsTemplate, { loading, reset, error: createTermsTemplateError }] =
    useCreateTermsTemplateMutation()

  const [formValues, setFormValues] = useState({
    name: "",
    annualRate: "",
    accrualInterval: "",
    incurrenceInterval: "",
    liquidationCvl: "",
    marginCallCvl: "",
    initialCvl: "",
    durationUnits: "",
    durationPeriod: "",
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
      const { data } = await createTermsTemplate({
        variables: {
          input: {
            name: formValues.name,
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
      if (data?.termsTemplateCreate.termsTemplate) {
        toast.success("Terms Template created successfully")
        if (refetch) refetch()
        setOpenCreateTermsTemplateDialog(false)
        router.push(`/terms-templates/${data.termsTemplateCreate.termsTemplate.termsId}`)
      } else {
        throw new Error("Failed to create Terms Template. Please try again.")
      }
    } catch (error) {
      console.error("Error creating Terms Template:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (createTermsTemplateError?.message) {
        setError(createTermsTemplateError.message)
      } else {
        setError("An unexpected error occurred. Please try again.")
      }
      toast.error("Failed to create Terms Template")
    }
  }

  const resetForm = () => {
    setFormValues({
      name: "",
      annualRate: "",
      accrualInterval: "",
      incurrenceInterval: "",
      liquidationCvl: "",
      marginCallCvl: "",
      initialCvl: "",
      durationUnits: "",
      durationPeriod: "",
    })
    setError(null)
    reset()
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
          <DialogTitle>Create Terms Template</DialogTitle>
          <DialogDescription>
            Create a new Terms Template by providing the required information
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="name">Template Name</Label>
            <Input
              id="name"
              name="name"
              type="text"
              required
              placeholder="Enter the template name"
              value={formValues.name}
              onChange={handleChange}
            />
          </div>
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
                    <option value="" disabled>
                      Select period
                    </option>
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
                  <option value="" disabled>
                    Select accrual interval
                  </option>
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
                  <option value="" disabled>
                    Select incurrence interval
                  </option>
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
              Create Terms Template
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
