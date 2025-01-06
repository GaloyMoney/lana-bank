/* eslint-disable no-empty-function */
"use client"

import { useState, useContext, createContext } from "react"
import { HiPlus } from "react-icons/hi"
import { usePathname } from "next/navigation"
import { toast } from "sonner"

import { CreateCustomerDialog } from "./customers/create"
import { CreateDepositDialog } from "./deposits/create"
import { WithdrawalInitiateDialog } from "./withdrawals/initiate"
import { CreateCreditFacilityDialog } from "./credit-facilities/create"
import { CreateUserDialog } from "./users/create"
import { CreateTermsTemplateDialog } from "./terms-templates/create"
import { CreateCommitteeDialog } from "./committees/create"
import CustomerSelector from "./customers/selector"

import { CreditFacility, Customer } from "@/lib/graphql/generated"
import { Button } from "@/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/ui/dropdown-menu"

const CreateButton = () => {
  const [createCustomer, setCreateCustomer] = useState(false)
  const [createDeposit, setCreateDeposit] = useState(false)
  const [createWithdrawal, setCreateWithdrawal] = useState(false)
  const [createFacility, setCreateFacility] = useState(false)
  const [openCreateUserDialog, setOpenCreateUserDialog] = useState(false)
  const [openCreateTermsTemplateDialog, setOpenCreateTermsTemplateDialog] =
    useState(false)
  const [openCreateCommitteeDialog, setOpenCreateCommitteeDialog] = useState(false)
  const [showMenu, setShowMenu] = useState(false)

  const { customer, setCustomer } = useCreateContext()
  const [openCustomerSelector, setOpenCustomerSelector] = useState(false)

  const pathName = usePathname()
  const userIsInCustomerDetailsPage = Boolean(pathName.match(/^\/customers\/.+$/))
  const setCustomerToNullIfNotInCustomerDetails = () => {
    if (!userIsInCustomerDetailsPage) setCustomer(null)
  }

  const menuItems = [
    {
      label: "Deposit",
      onClick: () => {
        if (!customer) setOpenCustomerSelector(true)
        setCreateDeposit(true)
      },
      dataTestId: "create-deposit-button",
    },
    {
      label: "Withdrawal",
      onClick: () => {
        if (!customer) setOpenCustomerSelector(true)
        setCreateWithdrawal(true)
      },
      dataTestId: "create-withdrawal-button",
    },
    {
      label: "Customer",
      onClick: () => setCreateCustomer(true),
      dataTestIds: "create-customer-button",
    },

    {
      label: "Credit Facility",
      onClick: () => {
        if (!customer) setOpenCustomerSelector(true)
        setCreateFacility(true)
      },
      dataTestId: "create-credit-facility-button",
    },
    {
      label: "User",
      onClick: () => setOpenCreateUserDialog(true),
      dataTestId: "create-user-button",
    },
    {
      label: "Terms Template",
      onClick: () => setOpenCreateTermsTemplateDialog(true),
      dataTestId: "create-terms-template-button",
    },
    {
      label: "Committee",
      onClick: () => setOpenCreateCommitteeDialog(true),
      dataTestId: "create-committee-button",
    },
  ]

  let creationType = ""
  if (createDeposit) creationType = "Deposit"
  if (createWithdrawal) creationType = "Withdrawal"
  if (createFacility) creationType = "Credit Facility"

  const decideCreation = () => {
    setShowMenu(false)
    if (pathName === "/customers") {
      setCreateCustomer(true)
    } else if (pathName === "/users") {
      setOpenCreateUserDialog(true)
    } else if (pathName === "/terms-templates") {
      setOpenCreateTermsTemplateDialog(true)
    } else if (pathName === "/committees") {
      setOpenCreateCommitteeDialog(true)
    } else if (pathName === "/deposits") {
      if (!customer) setOpenCustomerSelector(true)
      setCreateDeposit(true)
    } else if (pathName === "/withdrawals") {
      if (!customer) setOpenCustomerSelector(true)
      setCreateWithdrawal(true)
    } else if (pathName === "/credit-facilities") {
      if (!customer) setOpenCustomerSelector(true)
      setCreateFacility(true)
    } else if (pathName === "/disbursals") {
      toast.message("Disbursals can be initiated from credit facility page")
    } else {
      setShowMenu(true)
    }
  }

  return (
    <>
      <DropdownMenu
        open={showMenu}
        onOpenChange={(open) => {
          if (open) decideCreation()
          else setShowMenu(false)
        }}
      >
        <DropdownMenuTrigger asChild>
          <Button data-testid="global-create-button">
            <HiPlus className="h-4 w-4" />
            Create
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-36">
          {menuItems.map((item) => (
            <DropdownMenuItem
              data-testid={item.dataTestId}
              key={item.label}
              onClick={item.onClick}
              className="cursor-pointer"
            >
              {item.label}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      <CustomerSelector
        show={openCustomerSelector}
        setShow={setOpenCustomerSelector}
        setCustomer={setCustomer}
        onClose={() => {
          setCustomer(null)
          setCreateDeposit(false)
          setCreateWithdrawal(false)
          setCreateFacility(false)
        }}
        title={`Select customer for ${creationType}`}
      />

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

      {customer && (
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
          />
        </>
      )}
    </>
  )
}

export default CreateButton

type ICustomer = Customer | null
type IFacility = CreditFacility | null

type CreateContext = {
  customer: ICustomer
  facility: IFacility
  setCustomer: React.Dispatch<React.SetStateAction<ICustomer>>
  setFacility: React.Dispatch<React.SetStateAction<IFacility>>
}

const CreateContext = createContext<CreateContext>({
  customer: null,
  facility: null,
  setCustomer: () => {},
  setFacility: () => {},
})

export const CreateContextProvider: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  const [customer, setCustomer] = useState<ICustomer>(null)
  const [facility, setFacility] = useState<IFacility>(null)

  return (
    <CreateContext.Provider
      value={{
        customer,
        facility,
        setCustomer,
        setFacility,
      }}
    >
      {children}
    </CreateContext.Provider>
  )
}

export const useCreateContext = () => useContext(CreateContext)
