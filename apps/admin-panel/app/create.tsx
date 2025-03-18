/* eslint-disable no-empty-function */
"use client"

import { useState, useContext, createContext } from "react"
import { HiPlus } from "react-icons/hi"
import { usePathname } from "next/navigation"
import { useTranslations } from "next-intl"

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@lana/web/ui/tooltip"

import { Button } from "@lana/web/ui/button"

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@lana/web/ui/dropdown-menu"

import { CreateCustomerDialog } from "./customers/create"
import { CreateDepositDialog } from "./deposits/create"
import { WithdrawalInitiateDialog } from "./withdrawals/initiate"
import { CreateCreditFacilityDialog } from "./credit-facilities/create"

import { CreditFacilityPartialPaymentDialog } from "./credit-facilities/partial-payment"
import { CreateUserDialog } from "./users/create"
import { CreateTermsTemplateDialog } from "./terms-templates/create"
import { CreateCommitteeDialog } from "./committees/create"
import { CreditFacilityDisbursalInitiateDialog } from "./disbursals/create"

import {
  CreditFacility,
  Customer,
  CreditFacilityStatus,
  GetWithdrawalDetailsQuery,
  GetPolicyDetailsQuery,
  GetCommitteeDetailsQuery,
  TermsTemplateQuery,
  GetDisbursalDetailsQuery,
  KycLevel,
} from "@/lib/graphql/generated"

export const PATH_CONFIGS = {
  COMMITTEES: "/committees",
  COMMITTEE_DETAILS: /^\/committees\/[^/]+/,

  CREDIT_FACILITY_DETAILS: /^\/credit-facilities\/[^/]+/,

  CUSTOMERS: "/customers",
  CUSTOMER_DETAILS: /^\/customers\/[^/]+/,

  USERS: "/users",
  USER_DETAILS: /^\/users\/[^/]+/,

  TERMS_TEMPLATES: "/terms-templates",
  TERMS_TEMPLATE_DETAILS: /^\/terms-templates\/[^/]+/,

  WITHDRAWAL_DETAILS: /^\/withdrawals\/[^/]+/,
  POLICY_DETAILS: /^\/policies\/[^/]+/,
  DISBURSAL_DETAILS: /^\/disbursals\/[^/]+/,
}

const showCreateButton = (currentPath: string) => {
  const allowedPaths = Object.values(PATH_CONFIGS)
  return allowedPaths.some((path) => {
    if (typeof path === "string") {
      return path === currentPath
    } else if (path instanceof RegExp) {
      return path.test(currentPath)
    }
    return false
  })
}

const isItemAllowedOnCurrentPath = (
  allowedPaths: (string | RegExp)[],
  currentPath: string,
) => {
  return allowedPaths.some((path) => {
    if (typeof path === "string") {
      return path === currentPath
    } else if (path instanceof RegExp) {
      return path.test(currentPath)
    }
    return false
  })
}

type MenuItem = {
  label: string
  onClick: () => void
  dataTestId: string
  allowedPaths: (string | RegExp)[]
}

const CreateButton = () => {
  const t = useTranslations("CreateButton")

  const [createCustomer, setCreateCustomer] = useState(false)
  const [createDeposit, setCreateDeposit] = useState(false)
  const [createWithdrawal, setCreateWithdrawal] = useState(false)
  const [createFacility, setCreateFacility] = useState(false)
  const [initiateDisbursal, setInitiateDisbursal] = useState(false)
  const [makePayment, setMakePayment] = useState(false)
  const [openCreateUserDialog, setOpenCreateUserDialog] = useState(false)
  const [openCreateTermsTemplateDialog, setOpenCreateTermsTemplateDialog] =
    useState(false)
  const [openCreateCommitteeDialog, setOpenCreateCommitteeDialog] = useState(false)
  const [showMenu, setShowMenu] = useState(false)

  const { customer, facility, setCustomer } = useCreateContext()
  const pathName = usePathname()

  const userIsInCustomerDetailsPage = Boolean(pathName.match(/^\/customers\/.+$/))
  const setCustomerToNullIfNotInCustomerDetails = () => {
    if (!userIsInCustomerDetailsPage) setCustomer(null)
  }

  const isButtonDisabled = () => {
    if (
      PATH_CONFIGS.CUSTOMER_DETAILS.test(pathName) &&
      process.env.NODE_ENV !== "development"
    ) {
      return !customer || customer.level === KycLevel.NotKyced
    }
    if (PATH_CONFIGS.CREDIT_FACILITY_DETAILS.test(pathName)) {
      return !facility || facility.status !== CreditFacilityStatus.Active
    }
    return false
  }

  const getDisabledMessage = () => {
    if (pathName.includes("credit-facilities") && isButtonDisabled()) {
      return t("disabledMessages.creditFacilityMustBeActive")
    }
    return ""
  }

  const menuItems: MenuItem[] = [
    {
      label: t("menuItems.deposit"),
      onClick: () => {
        if (!customer) return
        setCreateDeposit(true)
      },
      dataTestId: "create-deposit-button",
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("menuItems.withdrawal"),
      onClick: () => {
        if (!customer) return
        setCreateWithdrawal(true)
      },
      dataTestId: "create-withdrawal-button",
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("menuItems.customer"),
      onClick: () => setCreateCustomer(true),
      dataTestId: "create-customer-button",
      allowedPaths: [PATH_CONFIGS.CUSTOMERS, PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("menuItems.creditFacility"),
      onClick: () => {
        if (!customer) return
        setCreateFacility(true)
      },
      dataTestId: "create-credit-facility-button",
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("menuItems.disbursal"),
      onClick: () => {
        if (!facility) return
        setInitiateDisbursal(true)
      },
      dataTestId: "initiate-disbursal-button",
      allowedPaths: [PATH_CONFIGS.CREDIT_FACILITY_DETAILS],
    },
    {
      label: t("menuItems.payment"),
      onClick: () => {
        if (!facility) return
        setMakePayment(true)
      },
      dataTestId: "make-payment-button",
      allowedPaths: [PATH_CONFIGS.CREDIT_FACILITY_DETAILS],
    },
    {
      label: t("menuItems.user"),
      onClick: () => setOpenCreateUserDialog(true),
      dataTestId: "create-user-button",
      allowedPaths: [PATH_CONFIGS.USERS, PATH_CONFIGS.USER_DETAILS],
    },
    {
      label: t("menuItems.termsTemplate"),
      onClick: () => setOpenCreateTermsTemplateDialog(true),
      dataTestId: "create-terms-template-button",
      allowedPaths: [PATH_CONFIGS.TERMS_TEMPLATES, PATH_CONFIGS.TERMS_TEMPLATE_DETAILS],
    },
    {
      label: t("menuItems.committee"),
      onClick: () => setOpenCreateCommitteeDialog(true),
      dataTestId: "create-committee-button",
      allowedPaths: [PATH_CONFIGS.COMMITTEES, PATH_CONFIGS.COMMITTEE_DETAILS],
    },
  ]

  const getAvailableMenuItems = () => {
    return menuItems.filter((item) =>
      isItemAllowedOnCurrentPath(item.allowedPaths, pathName),
    )
  }

  const decideCreation = () => {
    setShowMenu(false)
    const availableItems = getAvailableMenuItems()

    if (availableItems.length === 1) {
      availableItems[0].onClick()
      return
    }

    if (availableItems.length > 1) {
      setShowMenu(true)
    }
  }

  const availableItems = getAvailableMenuItems()
  if (!showCreateButton(pathName) || availableItems.length === 0) {
    return <div className="invisible w-[88px] h-[36px]" aria-hidden="true" />
  }

  const disabled = isButtonDisabled()
  const message = getDisabledMessage()

  return (
    <>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <div>
              <DropdownMenu
                open={showMenu && !disabled}
                onOpenChange={(open) => {
                  if (open && !disabled) decideCreation()
                  else setShowMenu(false)
                }}
              >
                <DropdownMenuTrigger asChild>
                  <Button
                    data-testid="global-create-button"
                    disabled={disabled}
                    tabIndex={-1}
                  >
                    <HiPlus className="h-4 w-4" />
                    {t("buttons.create")}
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-36">
                  {availableItems.map((item) => (
                    <DropdownMenuItem
                      key={item.label}
                      data-testid={item.dataTestId}
                      onClick={item.onClick}
                      className="cursor-pointer"
                    >
                      {item.label}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </TooltipTrigger>
          {disabled && message && (
            <TooltipContent>
              <p>{message}</p>
            </TooltipContent>
          )}
        </Tooltip>
      </TooltipProvider>

      <CreateCustomerDialog
        setOpenCreateCustomerDialog={setCreateCustomer}
        openCreateCustomerDialog={createCustomer}
      />

      <CreateUserDialog
        openCreateUserDialog={openCreateUserDialog}
        setOpenCreateUserDialog={setOpenCreateUserDialog}
      />

      <CreateTermsTemplateDialog
        openCreateTermsTemplateDialog={openCreateTermsTemplateDialog}
        setOpenCreateTermsTemplateDialog={setOpenCreateTermsTemplateDialog}
      />

      <CreateCommitteeDialog
        openCreateCommitteeDialog={openCreateCommitteeDialog}
        setOpenCreateCommitteeDialog={setOpenCreateCommitteeDialog}
      />

      {customer && customer.depositAccount && (
        <>
          <CreateDepositDialog
            openCreateDepositDialog={createDeposit}
            setOpenCreateDepositDialog={() => {
              setCustomerToNullIfNotInCustomerDetails()
              setCreateDeposit(false)
            }}
            depositAccountId={customer.depositAccount.depositAccountId}
          />

          <WithdrawalInitiateDialog
            openWithdrawalInitiateDialog={createWithdrawal}
            setOpenWithdrawalInitiateDialog={() => {
              setCustomerToNullIfNotInCustomerDetails()
              setCreateWithdrawal(false)
            }}
            depositAccountId={customer.depositAccount.depositAccountId}
          />

          <CreateCreditFacilityDialog
            openCreateCreditFacilityDialog={createFacility}
            setOpenCreateCreditFacilityDialog={() => {
              setCustomerToNullIfNotInCustomerDetails()
              setCreateFacility(false)
            }}
            customerId={customer.customerId}
            disbursalCreditAccountId={customer.depositAccount.depositAccountId}
          />
        </>
      )}

      {facility && (
        <>
          <CreditFacilityDisbursalInitiateDialog
            creditFacilityId={facility.creditFacilityId}
            openDialog={initiateDisbursal}
            setOpenDialog={() => {
              setInitiateDisbursal(false)
            }}
          />

          <CreditFacilityPartialPaymentDialog
            creditFacilityId={facility.creditFacilityId}
            openDialog={makePayment}
            setOpenDialog={() => {
              setMakePayment(false)
            }}
          />
        </>
      )}
    </>
  )
}

type ICustomer = Customer | null
type IFacility = CreditFacility | null
type ITermsTemplate = TermsTemplateQuery["termsTemplate"] | null
type IWithdraw = GetWithdrawalDetailsQuery["withdrawal"] | null
type IPolicy = GetPolicyDetailsQuery["policy"] | null
type ICommittee = GetCommitteeDetailsQuery["committee"] | null
type IDisbursal = GetDisbursalDetailsQuery["disbursal"] | null

type CreateContext = {
  customer: ICustomer
  facility: IFacility
  termsTemplate: ITermsTemplate
  withdraw: IWithdraw
  policy: IPolicy
  committee: ICommittee
  disbursal: IDisbursal

  setCustomer: React.Dispatch<React.SetStateAction<ICustomer>>
  setFacility: React.Dispatch<React.SetStateAction<IFacility>>
  setTermsTemplate: React.Dispatch<React.SetStateAction<ITermsTemplate>>
  setWithdraw: React.Dispatch<React.SetStateAction<IWithdraw>>
  setPolicy: React.Dispatch<React.SetStateAction<IPolicy>>
  setCommittee: React.Dispatch<React.SetStateAction<ICommittee>>
  setDisbursal: React.Dispatch<React.SetStateAction<IDisbursal>>
}

const CreateContext = createContext<CreateContext>({
  customer: null,
  facility: null,
  termsTemplate: null,
  withdraw: null,
  policy: null,
  committee: null,
  disbursal: null,

  setCustomer: () => {},
  setFacility: () => {},
  setTermsTemplate: () => {},
  setWithdraw: () => {},
  setPolicy: () => {},
  setCommittee: () => {},
  setDisbursal: () => {},
})

export const CreateContextProvider: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  const [customer, setCustomer] = useState<ICustomer>(null)
  const [facility, setFacility] = useState<IFacility>(null)
  const [termsTemplate, setTermsTemplate] = useState<ITermsTemplate>(null)
  const [withdraw, setWithdraw] = useState<IWithdraw>(null)
  const [policy, setPolicy] = useState<IPolicy>(null)
  const [committee, setCommittee] = useState<ICommittee>(null)
  const [disbursal, setDisbursal] = useState<IDisbursal>(null)

  return (
    <CreateContext.Provider
      value={{
        customer,
        facility,
        termsTemplate,
        withdraw,
        policy,
        committee,
        disbursal,

        setCustomer,
        setFacility,
        setTermsTemplate,
        setWithdraw,
        setPolicy,
        setCommittee,
        setDisbursal,
      }}
    >
      {children}
    </CreateContext.Provider>
  )
}

export const useCreateContext = () => useContext(CreateContext)

export default CreateButton
