import { gql } from "@apollo/client"
import React, { useEffect, useState } from "react"
import { toast } from "sonner"
import { useRouter } from "next/navigation"
import Link from "next/link"
import { PiPencilSimpleLineLight } from "react-icons/pi"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import { Input } from "@/components/primitive/input"
import { Label } from "@/components/primitive/label"
import {
  InterestInterval,
  Period,
  useCreditFacilityCreateMutation,
  useGetRealtimePriceUpdatesQuery,
  useTermsTemplatesQuery,
} from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import { Select } from "@/components/primitive/select"
import {
  formatInterval,
  formatPeriod,
  currencyConverter,
  calculateInitialCollateralRequired,
} from "@/lib/utils"
import { DetailItem } from "@/components/details"
import Balance from "@/components/balance/balance"

gql`
  mutation CreditFacilityCreate($input: CreditFacilityCreateInput!) {
    creditFacilityCreate(input: $input) {
      creditFacility {
        id
        creditFacilityId
      }
    }
  }
`

type CreateCreditFacilityDialogProps = {
  setOpenCreateCreditFacilityDialog: (isOpen: boolean) => void
  openCreateCreditFacilityDialog: boolean
  customerId: string
  refetch?: () => void
}

export const CreateCreditFacilityDialog: React.FC<CreateCreditFacilityDialogProps> = ({
  setOpenCreateCreditFacilityDialog,
  openCreateCreditFacilityDialog,
  customerId,
  refetch,
}) => {
  const router = useRouter()

  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const [isCustomTerms, setIsCustomTerms] = useState(false)
  const { data: termsTemplatesData, loading: termsTemplatesLoading } =
    useTermsTemplatesQuery()
  const [createCreditFacility, { loading, error, reset }] =
    useCreditFacilityCreateMutation()
  const [useTemplateTerms, setUseTemplateTerms] = useState(true)
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>("")

  const [formValues, setFormValues] = useState({
    facility: "0",
    annualRate: "",
    accrualInterval: "",
    incurrenceInterval: "",
    liquidationCvl: "",
    marginCallCvl: "",
    initialCvl: "",
    durationUnits: "",
    durationPeriod: "",
  })

  useEffect(() => {
    if (
      termsTemplatesData?.termsTemplates &&
      termsTemplatesData.termsTemplates.length > 0
    ) {
      const latestTemplate = termsTemplatesData.termsTemplates[0]
      setSelectedTemplateId(latestTemplate.id)
      setFormValues((prevValues) => ({
        ...prevValues,
        annualRate: latestTemplate.values.annualRate.toString(),
        accrualInterval: latestTemplate.values.accrualInterval,
        incurrenceInterval: latestTemplate.values.incurrenceInterval,
        liquidationCvl: latestTemplate.values.liquidationCvl.toString(),
        marginCallCvl: latestTemplate.values.marginCallCvl.toString(),
        initialCvl: latestTemplate.values.initialCvl.toString(),
        durationUnits: latestTemplate.values.duration.units.toString(),
        durationPeriod: latestTemplate.values.duration.period,
      }))
    }
  }, [termsTemplatesData])

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target
    setFormValues((prevValues) => ({
      ...prevValues,
      [name]: value,
    }))
    if (name === "facility") return
    setIsCustomTerms(true)
    setSelectedTemplateId("")
  }

  const handleTemplateChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const templateId = e.target.value
    setSelectedTemplateId(templateId)
    setIsCustomTerms(false)
    const selectedTemplate = termsTemplatesData?.termsTemplates.find(
      (t) => t.id === templateId,
    )
    if (selectedTemplate) {
      setFormValues((prevValues) => ({
        ...prevValues,
        annualRate: selectedTemplate.values.annualRate.toString(),
        accrualInterval: selectedTemplate.values.accrualInterval,
        incurrenceInterval: selectedTemplate.values.incurrenceInterval,
        liquidationCvl: selectedTemplate.values.liquidationCvl.toString(),
        marginCallCvl: selectedTemplate.values.marginCallCvl.toString(),
        initialCvl: selectedTemplate.values.initialCvl.toString(),
        durationUnits: selectedTemplate.values.duration.units.toString(),
        durationPeriod: selectedTemplate.values.duration.period,
      }))
    }
  }

  const handleCreateCreditFacility = async (event: React.FormEvent) => {
    event.preventDefault()
    const {
      facility,
      annualRate,
      accrualInterval,
      incurrenceInterval,
      liquidationCvl,
      marginCallCvl,
      initialCvl,
      durationUnits,
      durationPeriod,
    } = formValues

    if (
      !facility ||
      !annualRate ||
      !accrualInterval ||
      !incurrenceInterval ||
      !liquidationCvl ||
      !marginCallCvl ||
      !initialCvl ||
      !durationUnits ||
      !durationPeriod
    ) {
      toast.error("Please fill in all the fields.")
      return
    }

    try {
      await createCreditFacility({
        variables: {
          input: {
            customerId,
            facility: currencyConverter.usdToCents(Number(facility)),
            terms: {
              annualRate: parseFloat(annualRate),
              accrualInterval: accrualInterval as InterestInterval,
              incurrenceInterval: incurrenceInterval as InterestInterval,
              liquidationCvl: parseFloat(liquidationCvl),
              marginCallCvl: parseFloat(marginCallCvl),
              initialCvl: parseFloat(initialCvl),
              duration: {
                units: parseInt(durationUnits),
                period: durationPeriod as Period,
              },
            },
          },
        },
        onCompleted: (data) => {
          toast.success("Credit Facility created successfully")
          if (refetch) refetch()
          router.push(
            `/credit-facilities/${data?.creditFacilityCreate.creditFacility.creditFacilityId}`,
          )
        },
      })
    } catch (err) {
      console.error(err)
    }
  }

  const resetForm = () => {
    setUseTemplateTerms(true)
    setIsCustomTerms(false)
    if (
      termsTemplatesData?.termsTemplates &&
      termsTemplatesData.termsTemplates.length > 0
    ) {
      const latestTemplate = termsTemplatesData.termsTemplates[0]
      setSelectedTemplateId(latestTemplate.id)
      setFormValues({
        facility: "0",
        annualRate: latestTemplate.values.annualRate.toString(),
        accrualInterval: latestTemplate.values.accrualInterval,
        incurrenceInterval: latestTemplate.values.incurrenceInterval,
        liquidationCvl: latestTemplate.values.liquidationCvl.toString(),
        marginCallCvl: latestTemplate.values.marginCallCvl.toString(),
        initialCvl: latestTemplate.values.initialCvl.toString(),
        durationUnits: latestTemplate.values.duration.units.toString(),
        durationPeriod: latestTemplate.values.duration.period,
      })
    } else {
      setFormValues({
        facility: "0",
        annualRate: "",
        accrualInterval: "",
        incurrenceInterval: "",
        liquidationCvl: "",
        marginCallCvl: "",
        initialCvl: "",
        durationUnits: "",
        durationPeriod: "",
      })
    }
  }

  const handleCloseDialog = () => {
    setOpenCreateCreditFacilityDialog(false)
    resetForm()
    reset()
  }

  useEffect(() => {
    if (openCreateCreditFacilityDialog) {
      resetForm()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [openCreateCreditFacilityDialog])

  const collateralRequiredForDesiredFacility = calculateInitialCollateralRequired({
    amount: Number(formValues.facility) || 0,
    initialCvl: Number(formValues.initialCvl) || 0,
    priceInfo: priceInfo,
  })

  return (
    <Dialog open={openCreateCreditFacilityDialog} onOpenChange={handleCloseDialog}>
      <DialogContent className="max-w-[38rem]">
        <DialogHeader>
          <DialogTitle>Create Credit Facility</DialogTitle>
          <DialogDescription>
            Fill in the details to create a credit facility.
          </DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleCreateCreditFacility}>
          <div>
            <Label>Facility Amount</Label>
            <div className="flex items-center gap-1">
              <Input
                type="number"
                name="facility"
                value={formValues.facility}
                onChange={handleChange}
                placeholder="Enter the facility amount"
                min={0}
                required
              />
              <div className="p-1.5 bg-input-text rounded-md px-4">USD</div>
            </div>
          </div>
          {priceInfo && (
            <div className="text-sm ml-1 flex space-x-1 items-center">
              <Balance amount={collateralRequiredForDesiredFacility} currency="btc" />
              <div>collateral required (</div>
              <div>BTC/USD: </div>
              <Balance amount={priceInfo?.realtimePrice.usdCentsPerBtc} currency="usd" />
              <div>)</div>
            </div>
          )}
          {useTemplateTerms && termsTemplatesData?.termsTemplates.length === 0 ? (
            <div className="text-sm mt-1">
              No terms templates available,{" "}
              <Link
                className="text-primary hover:underline"
                href="/terms-templates?create=true"
              >
                create one here
              </Link>{" "}
              or manually specify the terms below.
            </div>
          ) : (
            <div>
              <Label>Terms Template</Label>
              <Select
                value={selectedTemplateId}
                onChange={handleTemplateChange}
                disabled={termsTemplatesLoading}
              >
                <option value="" disabled selected={isCustomTerms}>
                  Select from predefined terms template
                </option>
                {termsTemplatesData?.termsTemplates.map((template) => (
                  <option key={template.id} value={template.id}>
                    {template.name}
                  </option>
                ))}
              </Select>
            </div>
          )}
          {useTemplateTerms ? (
            <>
              <div
                onClick={() => setUseTemplateTerms(false)}
                className="mt-2 flex items-center space-x-2 ml-2 cursor-pointer text-sm hover:underline w-fit"
              >
                <div>Credit Facility Terms</div>
                <PiPencilSimpleLineLight className="w-5 h-5 cursor-pointer text-primary" />
              </div>
              <div className="grid grid-cols-2 gap-x-2">
                <DetailItem
                  label="Interest Rate (APR)"
                  value={formValues.annualRate + "%"}
                />
                <DetailItem label="Initial CVL (%)" value={formValues.initialCvl} />
                <DetailItem
                  label="Duration"
                  value={
                    String(formValues.durationUnits) +
                    " " +
                    formatPeriod(formValues.durationPeriod as Period)
                  }
                />
                <DetailItem
                  label="Margin Call CVL (%)"
                  value={formValues.marginCallCvl}
                />
                <DetailItem
                  label="Accrual Interval"
                  value={formatInterval(formValues.accrualInterval as InterestInterval)}
                />
                <DetailItem
                  label="Liquidation CVL (%)"
                  value={formValues.liquidationCvl}
                />
                <DetailItem
                  label="Incurrence Interval"
                  value={formatInterval(
                    formValues.incurrenceInterval as InterestInterval,
                  )}
                />
              </div>
            </>
          ) : (
            <>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Interest Rate (APR)</Label>
                  <Input
                    type="number"
                    name="annualRate"
                    value={formValues.annualRate}
                    onChange={handleChange}
                    placeholder="Enter the annual rate"
                    required
                  />
                </div>
                <div>
                  <Label>Initial CVL (%)</Label>
                  <Input
                    type="number"
                    name="initialCvl"
                    value={formValues.initialCvl}
                    onChange={handleChange}
                    placeholder="Enter the initial CVL"
                    required
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
                  <Label>Margin Call CVL (%)</Label>
                  <Input
                    type="number"
                    name="marginCallCvl"
                    value={formValues.marginCallCvl}
                    onChange={handleChange}
                    placeholder="Enter the margin call CVL"
                    required
                  />
                </div>
                <div>
                  <Label>Accrual Interval</Label>
                  <Select
                    name="accrualInterval"
                    value={formValues.accrualInterval}
                    onChange={handleChange}
                    required
                  >
                    <option value="" disabled>
                      Select accrual interval
                    </option>
                    {Object.values(InterestInterval).map((interval) => (
                      <option key={interval} value={interval}>
                        {formatInterval(interval)}
                      </option>
                    ))}
                  </Select>
                </div>
                <div>
                  <Label>Liquidation CVL (%)</Label>
                  <Input
                    type="number"
                    name="liquidationCvl"
                    value={formValues.liquidationCvl}
                    onChange={handleChange}
                    placeholder="Enter the liquidation CVL"
                    min={0}
                    required
                  />
                </div>
                <div>
                  <Label>Incurrence Interval</Label>
                  <Select
                    name="incurrenceInterval"
                    value={formValues.incurrenceInterval}
                    onChange={handleChange}
                    required
                  >
                    <option value="" disabled>
                      Select incurrence interval
                    </option>
                    {Object.values(InterestInterval).map((interval) => (
                      <option key={interval} value={interval}>
                        {formatInterval(interval)}
                      </option>
                    ))}
                  </Select>
                </div>
              </div>
            </>
          )}
          {error && <span className="text-destructive">{error.message}</span>}
          <DialogFooter className="mt-4">
            {!useTemplateTerms && (
              <Button
                type="button"
                onClick={() => setUseTemplateTerms(true)}
                variant="ghost"
              >
                Back
              </Button>
            )}
            <Button className="w-48" disabled={loading} type="submit" loading={loading}>
              Create Credit Facility
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
