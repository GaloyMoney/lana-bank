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
          interval
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

  const [name, setName] = useState<string>(termsTemplate.name)
  const [annualRate, setAnnualRate] = useState<string>(
    termsTemplate.values.annualRate.toString(),
  )
  const [interval, setInterval] = useState<InterestInterval>(
    termsTemplate.values.interval,
  )
  const [incurrenceInterval, setIncurrenceInterval] = useState<InterestInterval>(
    termsTemplate.values.incurrenceInterval,
  )
  const [duration, setDuration] = useState<{ period: Period; units: string }>({
    period: termsTemplate.values.duration.period,
    units: termsTemplate.values.duration.units.toString(),
  })
  const [liquidationCvl, setLiquidationCvl] = useState<string>(
    termsTemplate.values.liquidationCvl.toString(),
  )
  const [marginCallCvl, setMarginCallCvl] = useState<string>(
    termsTemplate.values.marginCallCvl.toString(),
  )
  const [initialCvl, setInitialCvl] = useState<string>(
    termsTemplate.values.initialCvl.toString(),
  )
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (openUpdateTermsTemplateDialog) {
      setName(termsTemplate.name)
      setAnnualRate(termsTemplate.values.annualRate.toString())
      setInterval(termsTemplate.values.interval)
      setIncurrenceInterval(termsTemplate.values.incurrenceInterval)
      setDuration({
        period: termsTemplate.values.duration.period,
        units: termsTemplate.values.duration.units.toString(),
      })
      setLiquidationCvl(termsTemplate.values.liquidationCvl.toString())
      setMarginCallCvl(termsTemplate.values.marginCallCvl.toString())
      setInitialCvl(termsTemplate.values.initialCvl.toString())
    }
  }, [openUpdateTermsTemplateDialog, termsTemplate])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const { data } = await updateTermsTemplate({
        variables: {
          input: {
            id: termsTemplate.termsId,
            annualRate,
            interval,
            incurrenceInterval,
            duration: {
              period: duration.period,
              units: parseInt(duration.units),
            },
            liquidationCvl,
            marginCallCvl,
            initialCvl,
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

  const resetStates = () => {
    setName(termsTemplate.name)
    setAnnualRate(termsTemplate.values.annualRate.toString())
    setInterval(termsTemplate.values.interval)
    setIncurrenceInterval(termsTemplate.values.incurrenceInterval)
    setDuration({
      period: termsTemplate.values.duration.period,
      units: termsTemplate.values.duration.units.toString(),
    })
    setLiquidationCvl(termsTemplate.values.liquidationCvl.toString())
    setMarginCallCvl(termsTemplate.values.marginCallCvl.toString())
    setInitialCvl(termsTemplate.values.initialCvl.toString())
    setError(null)
    reset()
  }

  return (
    <Dialog
      open={openUpdateTermsTemplateDialog}
      onOpenChange={(isOpen) => {
        setOpenUpdateTermsTemplateDialog(isOpen)
        if (!isOpen) {
          resetStates()
        }
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Update {name}</DialogTitle>
          <DialogDescription>
            Update the Terms Template by modifying the required information
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div>
            <Label htmlFor="annualRate">Annual Rate (%)</Label>
            <Input
              id="annualRate"
              type="number"
              required
              placeholder="Enter the annual rate"
              value={annualRate}
              onChange={(e) => setAnnualRate(e.target.value)}
            />
          </div>
          <div>
            <Label htmlFor="interval">Interval</Label>
            <Select
              id="interval"
              value={interval}
              onChange={(e) => setInterval(e.target.value as InterestInterval)}
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
              value={incurrenceInterval}
              onChange={(e) => setInterval(e.target.value as InterestInterval)}
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
            <Label>Duration</Label>
            <div className="flex gap-2">
              <Input
                type="number"
                value={duration.units}
                onChange={(e) => setDuration({ ...duration, units: e.target.value })}
                placeholder="Units"
                required
                className="w-1/2"
              />
              <Select
                value={duration.period}
                onChange={(e) =>
                  setDuration({ ...duration, period: e.target.value as Period })
                }
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
            <Label htmlFor="liquidationCvl">Liquidation CVL (%)</Label>
            <Input
              id="liquidationCvl"
              type="number"
              required
              placeholder="Enter the liquidation CVL"
              value={liquidationCvl}
              onChange={(e) => setLiquidationCvl(e.target.value)}
            />
          </div>
          <div>
            <Label htmlFor="marginCallCvl">Margin Call CVL (%)</Label>
            <Input
              id="marginCallCvl"
              type="number"
              required
              placeholder="Enter the margin call CVL"
              value={marginCallCvl}
              onChange={(e) => setMarginCallCvl(e.target.value)}
            />
          </div>
          <div>
            <Label htmlFor="initialCvl">Initial CVL (%)</Label>
            <Input
              id="initialCvl"
              type="number"
              required
              placeholder="Enter the initial CVL"
              value={initialCvl}
              onChange={(e) => setInitialCvl(e.target.value)}
            />
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
