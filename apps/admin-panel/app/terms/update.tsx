"use client"

import { gql } from "@apollo/client"
import React, { useState } from "react"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/primitive/dialog"
import { DetailItem, DetailsGroup } from "@/components/details"
import { Label } from "@/components/primitive/label"
import { Input } from "@/components/primitive/input"
import { Select } from "@/components/primitive/select"
import { Button } from "@/components/primitive/button"

import {
  DefaultTermsQuery,
  InterestInterval,
  Period,
  useDefaultTermsUpdateMutation,
} from "@/lib/graphql/generated"
import { formatInterval, formatPeriod } from "@/lib/utils"

gql`
  mutation DefaultTermsUpdate($input: DefaultTermsUpdateInput!) {
    defaultTermsUpdate(input: $input) {
      terms {
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

type UpdateDefaultTermDialogProps = {
  refetch?: () => void
  termsData?: DefaultTermsQuery | undefined
}

export const UpdateDefaultTermDialog: React.FC<
  React.PropsWithChildren<UpdateDefaultTermDialogProps>
> = ({ children, refetch, termsData }) => {
  const [interval, setInterval] = useState<InterestInterval | "">("")
  const [liquidationCvl, setLiquidationCvl] = useState<string>("")
  const [marginCallCvl, setMarginCallCvl] = useState<string>("")
  const [initialCvl, setInitialCvl] = useState<string>("")
  const [duration, setDuration] = useState<{ period: Period | ""; units: number | "" }>({
    period: "",
    units: "",
  })
  const [annualRate, setAnnualRate] = useState<number | "">("")

  const [updateDefaultTerm, { data, loading, error, reset }] =
    useDefaultTermsUpdateMutation()

  const handleUpdateDefaultTerm = async (event: React.FormEvent) => {
    event.preventDefault()

    if (
      annualRate === "" ||
      interval === "" ||
      duration.period === "" ||
      duration.units === "" ||
      liquidationCvl === "" ||
      marginCallCvl === "" ||
      initialCvl === ""
    ) {
      toast.error("Please fill in all the fields")
      return
    }

    try {
      await updateDefaultTerm({
        variables: {
          input: {
            annualRate: annualRate,
            interval: interval as InterestInterval,
            duration: {
              period: duration.period as Period,
              units: Number(duration.units),
            },
            liquidationCvl,
            marginCallCvl,
            initialCvl,
          },
        },
      })
      toast.success("Default Term updated")
      if (refetch) refetch()
    } catch (err) {
      console.error(err)
    }
  }

  const resetForm = () => {
    if (termsData && termsData.defaultTerms) {
      const { values } = termsData.defaultTerms
      setInterval(values.interval)
      setLiquidationCvl(values.liquidationCvl)
      setMarginCallCvl(values.marginCallCvl)
      setInitialCvl(values.initialCvl)
      setDuration({ period: values.duration.period, units: values.duration.units })
      setAnnualRate(values.annualRate)
    } else {
      setInterval("")
      setLiquidationCvl("")
      setMarginCallCvl("")
      setInitialCvl("")
      setDuration({ period: "", units: "" })
      setAnnualRate("")
    }
    reset()
  }

  return (
    <Dialog
      onOpenChange={(isOpen) => {
        if (isOpen) {
          resetForm()
        }
      }}
    >
      <DialogTrigger asChild>{children}</DialogTrigger>
      {data ? (
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Terms Updated</DialogTitle>
            <DialogDescription>Terms Details.</DialogDescription>
          </DialogHeader>
          <DetailsGroup>
            <DetailItem label="Terms ID" value={data.defaultTermsUpdate.terms.termsId} />
            <DetailItem
              label="Duration"
              value={
                String(data.defaultTermsUpdate.terms.values.duration.units) +
                " " +
                formatPeriod(data.defaultTermsUpdate.terms.values.duration.period)
              }
            />
            <DetailItem
              label="Interval"
              value={formatInterval(data.defaultTermsUpdate.terms.values.interval)}
            />
            <DetailItem
              label="Annual Rate"
              value={data.defaultTermsUpdate.terms.values.annualRate + "%"}
            />
            <DetailItem
              label="Initial CVL"
              value={data.defaultTermsUpdate.terms.values.initialCvl + "%"}
            />
            <DetailItem
              label="Margin Call CVL"
              value={data.defaultTermsUpdate.terms.values.marginCallCvl + "%"}
            />
            <DetailItem
              label="Liquidation CVL"
              value={data.defaultTermsUpdate.terms.values.liquidationCvl + "%"}
            />
          </DetailsGroup>
        </DialogContent>
      ) : (
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Update Terms</DialogTitle>
            <DialogDescription>
              Fill in the details to update the terms.
            </DialogDescription>
          </DialogHeader>
          <form className="flex flex-col gap-4" onSubmit={handleUpdateDefaultTerm}>
            <div>
              <Label>Initial CVL (%)</Label>
              <Input
                type="number"
                value={initialCvl}
                onChange={(e) => setInitialCvl(e.target.value)}
                placeholder="Enter the initial CVL"
                required
              />
            </div>
            <div>
              <Label>Margin Call CVL (%)</Label>
              <Input
                type="number"
                value={marginCallCvl}
                onChange={(e) => setMarginCallCvl(e.target.value)}
                placeholder="Enter the margin call CVL"
                required
              />
            </div>
            <div>
              <Label>Liquidation CVL (%)</Label>
              <Input
                type="number"
                value={liquidationCvl}
                onChange={(e) => setLiquidationCvl(e.target.value)}
                placeholder="Enter the liquidation CVL"
                min={0}
                required
              />
            </div>
            <div>
              <Label>Duration</Label>
              <div className="flex gap-2">
                <Input
                  type="number"
                  value={duration.units}
                  onChange={(e) =>
                    setDuration({
                      ...duration,
                      units: e.target.value === "" ? "" : parseInt(e.target.value),
                    })
                  }
                  placeholder="Duration"
                  min={0}
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
              <Label>Interval</Label>
              <Select
                value={interval}
                onChange={(e) => setInterval(e.target.value as InterestInterval)}
                required
              >
                <option value="" disabled>
                  Select interval
                </option>
                {Object.values(InterestInterval).map((interval) => (
                  <option key={interval} value={interval}>
                    {formatInterval(interval)}
                  </option>
                ))}
              </Select>
            </div>
            <div>
              <Label>Annual Rate (%)</Label>
              <Input
                type="number"
                value={annualRate}
                onChange={(e) =>
                  setAnnualRate(e.target.value === "" ? "" : parseFloat(e.target.value))
                }
                placeholder="Enter the annual rate"
                required
              />
            </div>
            {error && <span className="text-destructive">{error.message}</span>}
            <DialogFooter className="mt-4">
              <Button className="w-32" type="submit" loading={loading}>
                Submit
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      )}
    </Dialog>
  )
}
