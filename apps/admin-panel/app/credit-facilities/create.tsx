import React, { useEffect, useState } from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"
import { PiPencilSimpleLineLight } from "react-icons/pi"
import { useTranslations } from "next-intl"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Input } from "@lana/web/ui/input"
import { Label } from "@lana/web/ui/label"

import { Button } from "@lana/web/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"

import { useCreateContext } from "../create"

import {
  InterestInterval,
  Period,
  useCreditFacilityCreateMutation,
  useGetRealtimePriceUpdatesQuery,
  useTermsTemplatesQuery,
} from "@/lib/graphql/generated"
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
        customer {
          id
          creditFacilities {
            id
            creditFacilityId
            collateralizationState
            status
            createdAt
            balance {
              collateral {
                btcBalance
              }
              outstanding {
                usdBalance
              }
            }
          }
        }
      }
    }
  }
`

type CreateCreditFacilityDialogProps = {
  setOpenCreateCreditFacilityDialog: (isOpen: boolean) => void
  openCreateCreditFacilityDialog: boolean
  customerId: string
  disbursalCreditAccountId: string
}

const initialFormValues = {
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
}

export const CreateCreditFacilityDialog: React.FC<CreateCreditFacilityDialogProps> = ({
  setOpenCreateCreditFacilityDialog,
  openCreateCreditFacilityDialog,
  customerId,
  disbursalCreditAccountId,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.CreateCreditFacility")

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
      update: (cache) => {
        cache.modify({
          fields: {
            creditFacilities: (_, { DELETE }) => DELETE,
          },
        })
        cache.gc()
      },
    })

  const isLoading = loading || isNavigating

  const [useTemplateTerms, setUseTemplateTerms] = useState(true)
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>("")

  const [formValues, setFormValues] = useState(initialFormValues)

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
      toast.error(t("form.messages.fillAllFields"))
      return
    }

    try {
      await createCreditFacility({
        variables: {
          input: {
            disbursalCreditAccountId,
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
              interestDueDuration: {
                units: parseInt("0"),
                period: Period.Days,
              },
            },
          },
        },
        onCompleted: (data) => {
          if (data.creditFacilityCreate) {
            toast.success(t("form.messages.success"))
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
      setFormValues(initialFormValues)
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
      <DialogContent className="max-w-[40rem]">
        <div
          className="absolute -top-6 -left-[1px] bg-primary rounded-tl-md rounded-tr-md text-md px-2 py-1 text-secondary"
          style={{ width: "100.35%" }}
        >
          {t("dialog.customerInfo", { email: customer?.email })}
        </div>
        <DialogHeader>
          <DialogTitle>{t("dialog.title")}</DialogTitle>
          <DialogDescription>{t("dialog.description")}</DialogDescription>
        </DialogHeader>
        <form className="flex flex-col gap-4" onSubmit={handleCreateCreditFacility}>
          <div>
            <Label>{t("form.labels.facilityAmount")}</Label>
            <div className="flex items-center gap-1">
              <Input
                type="number"
                name="facility"
                value={formValues.facility}
                onChange={handleChange}
                placeholder={t("form.placeholders.facilityAmount")}
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
              <div>{t("form.messages.collateralRequired")} (</div>
              <div>{t("form.messages.btcUsdRate")} </div>
              <Balance amount={priceInfo?.realtimePrice.usdCentsPerBtc} currency="usd" />
              <div>)</div>
            </div>
          )}
          {useTemplateTerms && termsTemplatesData?.termsTemplates.length === 0 ? (
            <div className="text-sm mt-1">{t("form.messages.noTemplates")}</div>
          ) : (
            <div>
              <Label>{t("form.labels.termsTemplate")}</Label>
              <Select
                value={selectedTemplateId}
                onValueChange={handleTemplateChange}
                disabled={termsTemplatesLoading}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t("form.placeholders.termsTemplate")} />
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
              <button
                type="button"
                onClick={() => setUseTemplateTerms(false)}
                className="mt-2 flex items-center space-x-2 ml-2 cursor-pointer text-sm hover:underline w-fit"
              >
                <div>{t("form.labels.creditFacilityTerms")}</div>
                <PiPencilSimpleLineLight className="w-5 h-5 cursor-pointer text-primary" />
              </button>
              <DetailsGroup
                layout="horizontal"
                className="grid auto-rows-fr sm:grid-cols-2"
              >
                <DetailItem
                  label={t("form.labels.interestRate")}
                  value={formValues.annualRate + "%"}
                />
                <DetailItem
                  label={t("form.labels.initialCvl")}
                  value={formValues.initialCvl}
                />
                <DetailItem
                  label={t("form.labels.duration")}
                  value={
                    String(formValues.durationUnits) +
                    " " +
                    formatPeriod(formValues.durationPeriod as Period)
                  }
                />
                <DetailItem
                  label={t("form.labels.marginCallCvl")}
                  value={formValues.marginCallCvl}
                />
                <DetailItem
                  label={t("form.labels.accrualInterval")}
                  value={formatInterval(formValues.accrualInterval as InterestInterval)}
                />
                <DetailItem
                  label={t("form.labels.liquidationCvl")}
                  value={formValues.liquidationCvl}
                />
                <DetailItem
                  label={t("form.labels.incurrenceInterval")}
                  value={formatInterval(
                    formValues.incurrenceInterval as InterestInterval,
                  )}
                />
                <DetailItem
                  label={t("form.labels.structuringFeeRate")}
                  value={formValues.oneTimeFeeRate}
                />
              </DetailsGroup>
            </>
          ) : (
            <>
              <div className="grid auto-rows-fr sm:grid-cols-2 gap-4">
                <div>
                  <Label>{t("form.labels.interestRate")}</Label>
                  <Input
                    type="number"
                    name="annualRate"
                    value={formValues.annualRate}
                    onChange={handleChange}
                    placeholder={t("form.placeholders.annualRate")}
                    required
                  />
                </div>
                <div>
                  <Label>{t("form.labels.initialCvl")}</Label>
                  <Input
                    type="number"
                    name="initialCvl"
                    value={formValues.initialCvl}
                    onChange={handleChange}
                    placeholder={t("form.placeholders.initialCvl")}
                    required
                  />
                </div>
                <div>
                  <Label>{t("form.labels.duration")}</Label>
                  <div className="flex gap-2">
                    <Input
                      type="number"
                      name="durationUnits"
                      value={formValues.durationUnits}
                      onChange={handleChange}
                      placeholder={t("form.placeholders.duration")}
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
                        <SelectValue placeholder={t("form.placeholders.selectPeriod")} />
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
                  <Label>{t("form.labels.marginCallCvl")}</Label>
                  <Input
                    type="number"
                    name="marginCallCvl"
                    value={formValues.marginCallCvl}
                    onChange={handleChange}
                    placeholder={t("form.placeholders.marginCallCvl")}
                    required
                  />
                </div>
                <div>
                  <Label>{t("form.labels.accrualInterval")}</Label>
                  <Select
                    value={formValues.accrualInterval}
                    onValueChange={(value) =>
                      handleChange({
                        target: { name: "accrualInterval", value },
                      } as React.ChangeEvent<HTMLSelectElement>)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t("form.placeholders.accrualInterval")} />
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
                  <Label>{t("form.labels.liquidationCvl")}</Label>
                  <Input
                    type="number"
                    name="liquidationCvl"
                    value={formValues.liquidationCvl}
                    onChange={handleChange}
                    placeholder={t("form.placeholders.liquidationCvl")}
                    min={0}
                    required
                  />
                </div>
                <div>
                  <Label>{t("form.labels.incurrenceInterval")}</Label>
                  <Select
                    value={formValues.incurrenceInterval}
                    onValueChange={(value) =>
                      handleChange({
                        target: { name: "incurrenceInterval", value },
                      } as React.ChangeEvent<HTMLSelectElement>)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue
                        placeholder={t("form.placeholders.incurrenceInterval")}
                      />
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
                  <Label>{t("form.labels.structuringFeeRate")}</Label>
                  <Input
                    type="number"
                    name="oneTimeFeeRate"
                    value={formValues.oneTimeFeeRate}
                    onChange={handleChange}
                    placeholder={t("form.placeholders.structuringFeeRate")}
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
                {t("form.buttons.back")}
              </Button>
            )}
            <Button
              disabled={isLoading}
              type="submit"
              loading={isLoading}
              data-testid="create-credit-facility-submit"
            >
              {t("form.buttons.create")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
