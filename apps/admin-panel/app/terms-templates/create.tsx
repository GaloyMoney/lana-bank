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

  const [name, setName] = useState<string>("")
  const [annualRate, setAnnualRate] = useState<string>("")
  const [interval, setInterval] = useState<InterestInterval | "">("")
  const [duration, setDuration] = useState<{ period: Period | ""; units: string }>({
    period: "",
    units: "",
  })
  const [liquidationCvl, setLiquidationCvl] = useState<string>("")
  const [marginCallCvl, setMarginCallCvl] = useState<string>("")
  const [initialCvl, setInitialCvl] = useState<string>("")
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      const { data } = await createTermsTemplate({
        variables: {
          input: {
            name,
            annualRate,
            interval: interval as InterestInterval,
            duration: {
              period: duration.period as Period,
              units: parseInt(duration.units),
            },
            liquidationCvl,
            marginCallCvl,
            initialCvl,
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

  const resetStates = () => {
    setName("")
    setAnnualRate("")
    setInterval("")
    setDuration({ period: "", units: "" })
    setLiquidationCvl("")
    setMarginCallCvl("")
    setInitialCvl("")
    setError(null)
    reset()
  }

  return (
    <Dialog
      open={openCreateTermsTemplateDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateTermsTemplateDialog(isOpen)
        if (!isOpen) {
          resetStates()
        }
      }}
    >
      <DialogContent>
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
              type="text"
              required
              placeholder="Enter the template name"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
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
              <option value="" disabled>
                Select interval
              </option>
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
              Create Terms Template
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
