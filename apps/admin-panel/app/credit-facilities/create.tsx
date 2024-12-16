import React, { useEffect, useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { PiPencilSimpleLineLight } from "react-icons/pi"

import { useCreateContext } from "../create"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/ui/dialog"
import { Input } from "@/ui/input"
import { Label } from "@/ui/label"
import {
  AllActionsDocument,
  CreditFacilitiesDocument,
  InterestInterval,
  Period,
  useCreditFacilityCreateMutation,
  useGetRealtimePriceUpdatesQuery,
  useTermsTemplatesQuery,
} from "@/lib/graphql/generated"
import { Button } from "@/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/ui/select"
import {
  formatInterval,
  formatPeriod,
  currencyConverter,
  calculateInitialCollateralRequired,
} from "@/lib/utils"
import { DetailItem, DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"
import { useModalNavigation } from "@/hooks/use-modal-navigation"
import { Satoshis } from "@/types"

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
}

export const CreateCreditFacilityDialog: React.FC<CreateCreditFacilityDialogProps> = ({
  setOpenCreateCreditFacilityDialog,
  openCreateCreditFacilityDialog,
  customerId,
}) => {
  const { navigate, isNavigating } = useModalNavigation({
    closeModal: () => setOpenCreateCreditFacilityDialog(false),
  })
  const { customer } = useCreateContext()

  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const { data: termsTemplatesData, loading: termsTemplatesLoading } =
    useTermsTemplatesQuery()
  const [createCreditFacility, { loading, error, reset }] =
    useCreditFacilityCreateMutation({
      refetchQueries: [CreditFacilitiesDocument, AllActionsDocument],
      awaitRefetchQueries: true,
    })

  const isLoading = loading || isNavigating

  const [useTemplateTerms, setUseTemplateTerms] = useState(true)
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>("")

  // State and handlers are unchanged...
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
    oneTimeFeeRate: "",
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
        oneTimeFeeRate: latestTemplate.values.oneTimeFeeRate.toString(),
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
    setSelectedTemplateId("")
  }

  const handleTemplateChange = (templateId: string) => {
    setSelectedTemplateId(templateId)
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
        oneTimeFeeRate: selectedTemplate.values.oneTimeFeeRate.toString(),
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
      oneTimeFeeRate,
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
      !durationPeriod ||
      !oneTimeFeeRate
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
              oneTimeFeeRate: parseFloat(oneTimeFeeRate),
              duration: {
                units: parseInt(durationUnits),
                period: durationPeriod as Period,
              },
            },
          },
        },
        onCompleted: (data) => {
          if (data.creditFacilityCreate) {
            toast.success("Credit Facility created successfully")
            navigate(
              `/credit-facilities/${data?.creditFacilityCreate.creditFacility.creditFacilityId}`,
            )
          }
        },
      })
    } catch (err) {
      console.error(err)
    }
  }

  const resetForm = () => {
    setUseTemplateTerms(true)
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
        oneTimeFeeRate: latestTemplate.values.oneTimeFeeRate?.toString(),
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
        oneTimeFeeRate: "",
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
        <div
          className="absolute -top-6 -left-[1px] bg-primary rounded-tl-md rounded-tr-md text-md px-2 py-1 text-secondary"
          style={{ width: "100.35%" }}
        >
          Creating credit facility for {customer?.email}
        </div>
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
                data-testid="facility-amount-input"
                required
              />
              <div className="p-1.5 bg-input-text rounded-md px-4">USD</div>
            </div>
          </div>
          {priceInfo && (
            <div className="text-sm ml-1 flex space-x-1 items-center">
              <Balance
                amount={collateralRequiredForDesiredFacility as Satoshis}
                currency="btc"
              />
              <div>collateral required (</div>
              <div>BTC/USD: </div>
              <Balance amount={priceInfo?.realtimePrice.usdCentsPerBtc} currency="usd" />
              <div>)</div>
            </div>
          )}
          {useTemplateTerms && termsTemplatesData?.termsTemplates.length === 0 ? (
            <div className="text-sm mt-1">
              No terms templates available please create one or manually specify the terms
              below.
            </div>
          ) : (
            <div>
              <Label>Terms Template</Label>
              <Select
                value={selectedTemplateId}
                onValueChange={handleTemplateChange}
                disabled={termsTemplatesLoading}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select from predefined terms template" />
                </SelectTrigger>
                <SelectContent>
                  {termsTemplatesData?.termsTemplates.map((template) => (
                    <SelectItem key={template.id} value={template.id}>
                      {template.name}
                    </SelectItem>
                  ))}
                </SelectContent>
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
              <DetailsGroup layout="horizontal" className="grid grid-cols-2 gap-x-2">
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
                <DetailItem
                  label="One-Time Fee Rate (%)"
                  value={formValues.oneTimeFeeRate}
                />
              </DetailsGroup>
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
                      value={formValues.durationPeriod}
                      onValueChange={(value) =>
                        handleChange({
                          target: { name: "durationPeriod", value },
                        } as React.ChangeEvent<HTMLSelectElement>)
                      }
                    >
                      <SelectTrigger>
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
                    value={formValues.accrualInterval}
                    onValueChange={(value) =>
                      handleChange({
                        target: { name: "accrualInterval", value },
                      } as React.ChangeEvent<HTMLSelectElement>)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select accrual interval" />
                    </SelectTrigger>
                    <SelectContent>
                      {Object.values(InterestInterval).map((interval) => (
                        <SelectItem key={interval} value={interval}>
                          {formatInterval(interval)}
                        </SelectItem>
                      ))}
                    </SelectContent>
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
                    value={formValues.incurrenceInterval}
                    onValueChange={(value) =>
                      handleChange({
                        target: { name: "incurrenceInterval", value },
                      } as React.ChangeEvent<HTMLSelectElement>)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select incurrence interval" />
                    </SelectTrigger>
                    <SelectContent>
                      {Object.values(InterestInterval).map((interval) => (
                        <SelectItem key={interval} value={interval}>
                          {formatInterval(interval)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label>One-Time Fee Rate (%)</Label>
                  <Input
                    type="number"
                    name="oneTimeFeeRate"
                    value={formValues.oneTimeFeeRate}
                    onChange={handleChange}
                    placeholder="Enter the one-time fee rate"
                    min={0}
                    required
                  />
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
            <Button
              className="w-48"
              disabled={isLoading}
              type="submit"
              loading={isLoading}
              data-testid="create-credit-facility-submit"
            >
              Create Credit Facility
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
