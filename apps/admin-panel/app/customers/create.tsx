import React, { useState } from "react"
import { toast } from "sonner"
import { gql } from "@apollo/client"
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
import { Button } from "@lana/web/ui/button"
import { Label } from "@lana/web/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@lana/web/ui/select"
import { RadioGroup, RadioGroupItem } from "@lana/web/ui/radio-group"

import { useCustomerCreateMutation, CustomerType } from "@/lib/graphql/generated"
import { useModalNavigation } from "@/hooks/use-modal-navigation"

gql`
  mutation CustomerCreate($input: CustomerCreateInput!) {
    customerCreate(input: $input) {
      customer {
        id
        customerId
        publicId
        email
        kycVerification
        level
        applicantId
      }
    }
  }
`

type FormData = {
  email: string
  telegramHandle: string
  customerType: CustomerType
}

type CreateCustomerDialogProps = {
  setOpenCreateCustomerDialog: (isOpen: boolean) => void
  openCreateCustomerDialog: boolean
}

const InitialFormData: FormData = {
  email: "",
  telegramHandle: "",
  customerType: CustomerType.Individual,
}

type FormProps = {
  formData: FormData
  handleInputChange: (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => void
  handleSubmit: (e: React.FormEvent) => void
  isLoading: boolean
  error: string | null
  setCurrentStep: (step: "details" | "confirmation") => void
  t: ReturnType<typeof useTranslations<"Customers.create">>
  setFormData?: React.Dispatch<React.SetStateAction<FormData>>
}

const DetailsForm = ({
  formData,
  handleInputChange,
  handleSubmit,
  isLoading,
  error,
  t,
  setFormData,
}: FormProps) => (
  <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
    <div>
      <Label htmlFor="email">{t("emailLabel")}</Label>
      <Input
        id="email"
        name="email"
        type="email"
        required
        data-testid="customer-create-email"
        placeholder={t("emailPlaceholder")}
        value={formData.email}
        onChange={handleInputChange}
        disabled={isLoading}
      />
    </div>
    <div>
      <Label htmlFor="telegramHandle">{t("telegramLabel")}</Label>
      <Input
        id="telegramHandle"
        name="telegramHandle"
        type="text"
        required
        data-testid="customer-create-telegram-id"
        placeholder={t("telegramPlaceholder")}
        value={formData.telegramHandle}
        onChange={handleInputChange}
        disabled={isLoading}
      />
    </div>
    <div>
      <Label>{t("customerTypeLabel")}</Label>
      <RadioGroup
        className="flex gap-4 mt-2"
        value={
          formData.customerType === CustomerType.Individual ? "INDIVIDUAL" : "COMPANY"
        }
        onValueChange={(value) => {
          if (setFormData) {
            if (value === "INDIVIDUAL") {
              setFormData((prev) => ({
                ...prev,
                customerType: CustomerType.Individual,
              }))
            } else {
              setFormData((prev) => ({
                ...prev,
                customerType: CustomerType.NonDomiciledCompany,
              }))
            }
          }
        }}
        disabled={isLoading}
      >
        <div className="flex items-center">
          <RadioGroupItem
            id="individual"
            value="INDIVIDUAL"
            data-testid="customer-type-individual"
          />
          <Label htmlFor="individual" className="ml-2 text-sm font-normal">
            {t("individualLabel")}
          </Label>
        </div>
        <div className="flex items-center">
          <RadioGroupItem
            id="company"
            value="COMPANY"
            data-testid="customer-type-company"
          />
          <Label htmlFor="company" className="ml-2 text-sm font-normal">
            {t("companyLabel")}
          </Label>
        </div>
      </RadioGroup>
    </div>

    <div>
      {formData.customerType !== CustomerType.Individual ? (
        <div>
          <Label htmlFor="companyType">{t("companyTypeLabel")}</Label>
          <Select
            value={formData.customerType}
            onValueChange={(value) => {
              if (setFormData) {
                setFormData((prev) => ({
                  ...prev,
                  customerType: value as CustomerType,
                }))
              }
            }}
            disabled={isLoading}
          >
            <SelectTrigger
              id="companyType"
              data-testid="company-type-select"
              className="w-full"
            >
              <SelectValue placeholder={t("selectCompanyType")} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={CustomerType.GovernmentEntity}>
                {t("governmentEntityLabel")}
              </SelectItem>
              <SelectItem value={CustomerType.PrivateCompany}>
                {t("privateCompanyLabel")}
              </SelectItem>
              <SelectItem value={CustomerType.Bank}>{t("bankLabel")}</SelectItem>
              <SelectItem value={CustomerType.FinancialInstitution}>
                {t("financialInstitutionLabel")}
              </SelectItem>
              <SelectItem value={CustomerType.ForeignAgencyOrSubsidiary}>
                {t("foreignAgencyLabel")}
              </SelectItem>
              <SelectItem value={CustomerType.NonDomiciledCompany}>
                {t("nonDomiciledCompanyLabel")}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      ) : (
        <div aria-hidden="true" className="invisible">
          <Label htmlFor="placeholder">{t("companyTypeLabel")}</Label>
          <div className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
            &nbsp;
          </div>
        </div>
      )}
    </div>

    {error && <p className="text-destructive">{error}</p>}
    <DialogFooter>
      <Button
        type="submit"
        loading={isLoading}
        data-testid="customer-create-submit-button"
      >
        {t("reviewButton")}
      </Button>
    </DialogFooter>
  </form>
)

const ConfirmationForm = ({
  formData,
  handleSubmit,
  isLoading,
  error,
  setCurrentStep,
  t,
}: FormProps) => (
  <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
    <input
      type="text"
      className="sr-only"
      autoFocus
      onKeyDown={(e) => {
        if (e.key === "Backspace") {
          e.preventDefault()
          setCurrentStep("details")
        }
      }}
    />
    <div>
      <Label>{t("emailLabel")}</Label>
      <p>{formData.email}</p>
    </div>
    <div>
      <Label>{t("telegramLabel")}</Label>
      <p>{formData.telegramHandle}</p>
    </div>
    <div>
      <Label>{t("customerTypeLabel")}</Label>
      <p>{getCustomerTypeDisplay(formData.customerType, t)}</p>
    </div>
    {error && <p className="text-destructive">{error}</p>}
    <DialogFooter>
      <Button
        variant="ghost"
        onClick={() => setCurrentStep("details")}
        disabled={isLoading}
        type="button"
      >
        {t("backButton")}
      </Button>
      <Button
        type="submit"
        loading={isLoading}
        data-testid="customer-create-submit-button"
      >
        {t("confirmButton")}
      </Button>
    </DialogFooter>
  </form>
)

// Helper function to get display text for customer type
const getCustomerTypeDisplay = (
  customerType: CustomerType,
  t: ReturnType<typeof useTranslations<"Customers.create">>,
) => {
  switch (customerType) {
    case CustomerType.Individual:
      return t("individualLabel")
    case CustomerType.GovernmentEntity:
      return t("governmentEntityLabel")
    case CustomerType.PrivateCompany:
      return t("privateCompanyLabel")
    case CustomerType.Bank:
      return t("bankLabel")
    case CustomerType.FinancialInstitution:
      return t("financialInstitutionLabel")
    case CustomerType.ForeignAgencyOrSubsidiary:
      return t("foreignAgencyLabel")
    case CustomerType.NonDomiciledCompany:
      return t("nonDomiciledCompanyLabel")
    default:
      return customerType
  }
}

export const CreateCustomerDialog: React.FC<CreateCustomerDialogProps> = ({
  setOpenCreateCustomerDialog,
  openCreateCustomerDialog,
}) => {
  const t = useTranslations("Customers.create")

  const closeDialog = () => {
    setOpenCreateCustomerDialog(false)
    resetForm()
  }

  const { navigate, isNavigating } = useModalNavigation({
    closeModal: closeDialog,
  })

  const [createCustomer, { loading, error: createCustomerError }] =
    useCustomerCreateMutation({
      update: (cache) => {
        cache.modify({
          fields: {
            customers: (_, { DELETE }) => DELETE,
          },
        })
        cache.gc()
      },
    })

  const isLoading = loading || isNavigating
  const [formData, setFormData] = useState<FormData>(InitialFormData)
  const [error, setError] = useState<string | null>(null)
  const [currentStep, setCurrentStep] = useState<"details" | "confirmation">("details")

  const handleInputChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>,
  ) => {
    const { name, value } = e.target
    setFormData((prev) => ({
      ...prev,
      [name]: value,
    }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (currentStep === "details") {
      setCurrentStep("confirmation")
      return
    }

    try {
      await createCustomer({
        variables: {
          input: formData,
        },
        onCompleted: (data) => {
          if (data?.customerCreate.customer) {
            toast.success(t("successMessage"))
            navigate(`/customers/${data.customerCreate.customer.publicId}`)
          } else {
            throw new Error(t("failedToCreate"))
          }
        },
      })
    } catch (error) {
      console.error("Error creating customer:", error)
      if (error instanceof Error) {
        setError(error.message)
      } else if (createCustomerError?.message) {
        setError(createCustomerError.message)
      } else {
        setError(t("unexpectedError"))
      }
      toast.error(t("errorMessage"))
    }
  }

  const resetForm = () => {
    setFormData(InitialFormData)
    setError(null)
    setCurrentStep("details")
  }

  return (
    <Dialog
      open={openCreateCustomerDialog}
      onOpenChange={(isOpen) => {
        setOpenCreateCustomerDialog(isOpen)
        if (!isOpen) resetForm()
      }}
    >
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {currentStep === "details" ? t("title") : t("confirmTitle")}
          </DialogTitle>
          <DialogDescription>
            {currentStep === "details" ? t("description") : t("confirmDescription")}
          </DialogDescription>
        </DialogHeader>
        {currentStep === "details" ? (
          <DetailsForm
            formData={formData}
            handleInputChange={handleInputChange}
            handleSubmit={(e) => {
              e.preventDefault()
              setCurrentStep("confirmation")
            }}
            isLoading={isLoading}
            error={error}
            setCurrentStep={setCurrentStep}
            t={t}
            setFormData={setFormData}
          />
        ) : (
          <ConfirmationForm
            formData={formData}
            handleInputChange={handleInputChange}
            handleSubmit={handleSubmit}
            isLoading={isLoading}
            error={error}
            setCurrentStep={setCurrentStep}
            t={t}
            setFormData={setFormData}
          />
        )}
      </DialogContent>
    </Dialog>
  )
}
