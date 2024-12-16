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
} from "@/ui/dialog"
import {
  useCreateTermsTemplateMutation,
  InterestInterval,
  Period,
  TermsTemplatesDocument,
} from "@/lib/graphql/generated"
import { Input } from "@/ui/input"
import { Button } from "@/ui/button"
import { Label } from "@/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/ui/select"
import { formatInterval, formatPeriod } from "@/lib/utils"
import { useModalNavigation } from "@/hooks/use-modal-navigation"

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
}) => {
  const { navigate, isNavigating } = useModalNavigation({
    closeModal: () => setOpenCreateTermsTemplateDialog(false),
  })

  const [createTermsTemplate, { loading, reset, error: createTermsTemplateError }] =
    useCreateTermsTemplateMutation({
      refetchQueries: [TermsTemplatesDocument],
    })

  const isLoading = loading || isNavigating

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
            accrualInterval: formValues.accrualInterval as InterestInterval,
            incurrenceInterval: formValues.incurrenceInterval as InterestInterval,
            duration: {
              period: formValues.durationPeriod as Period,
              units: parseInt(formValues.durationUnits),
            },
            liquidationCvl: formValues.liquidationCvl,
            marginCallCvl: formValues.marginCallCvl,
            initialCvl: formValues.initialCvl,
            oneTimeFeeRate: formValues.oneTimeFeeRate,
          },
        },
        onCompleted: (data) => {
          toast.success("Terms Template created successfully")
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
      oneTimeFeeRate: "",
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
              disabled={isLoading}
              data-testid="terms-template-name-input"
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
                  disabled={isLoading}
                  data-testid="terms-template-annual-rate-input"
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
                    disabled={isLoading}
                    className="w-1/2"
                    data-testid="terms-template-duration-units-input"
                  />
                  <Select
                    value={formValues.durationPeriod}
                    onValueChange={(value) =>
                      handleChange({
                        target: { name: "durationPeriod", value },
                      } as React.ChangeEvent<HTMLSelectElement>)
                    }
                  >
                    <SelectTrigger data-testid="terms-template-duration-period-select">
                      <SelectValue placeholder="Select period" />
                    </SelectTrigger>
                    <SelectContent>
                      {Object.values(Period).map((period) => (
                        <SelectItem key={period} value={period}>
                          {formatPeriod(period)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div>
                <Label htmlFor="accrualInterval">Accrual Interval</Label>
                <Select
                  value={formValues.accrualInterval}
                  onValueChange={(value) =>
                    handleChange({
                      target: { name: "accrualInterval", value },
                    } as React.ChangeEvent<HTMLSelectElement>)
                  }
                >
                  <SelectTrigger data-testid="terms-template-accrual-interval-select">
                    <SelectValue placeholder="Select accrual interval" />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.values(InterestInterval).map((int) => (
                      <SelectItem key={int} value={int}>
                        {formatInterval(int)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label htmlFor="incurrenceInterval">Incurrence Interval</Label>
                <Select
                  value={formValues.incurrenceInterval}
                  onValueChange={(value) =>
                    handleChange({
                      target: { name: "incurrenceInterval", value },
                    } as React.ChangeEvent<HTMLSelectElement>)
                  }
                >
                  <SelectTrigger data-testid="terms-template-incurrence-interval-select">
                    <SelectValue placeholder="Select incurrence interval" />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.values(InterestInterval).map((int) => (
                      <SelectItem key={int} value={int}>
                        {formatInterval(int)}
                      </SelectItem>
                    ))}
                  </SelectContent>
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
                  disabled={isLoading}
                  data-testid="terms-template-initial-cvl-input"
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
                  disabled={isLoading}
                  data-testid="terms-template-margin-call-cvl-input"
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
                  disabled={isLoading}
                  data-testid="terms-template-liquidation-cvl-input"
                />
              </div>
              <div>
                <Label htmlFor="oneTimeFeeRate">One-Time Fee Rate (%)</Label>
                <Input
                  id="oneTimeFeeRate"
                  name="oneTimeFeeRate"
                  type="number"
                  required
                  placeholder="Enter the one-time fee rate"
                  value={formValues.oneTimeFeeRate}
                  onChange={handleChange}
                  disabled={isLoading}
                  data-testid="terms-template-one-time-fee-rate-input"
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
              Create Terms Template
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
