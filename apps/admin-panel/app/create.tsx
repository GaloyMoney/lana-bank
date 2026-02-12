/* eslint-disable no-empty-function */
"use client"

import { useState, useContext, createContext, useMemo } from "react"
import { HiPlus } from "react-icons/hi"
import { usePathname, useRouter } from "next/navigation"
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

import { getUTCYear } from "@lana/web/utils"

import { CreateProspectDialog } from "./prospects/create"
import { CreateDepositDialog } from "./deposits/create"
import { WithdrawalInitiateDialog } from "./withdrawals/initiate"
import { CreateCreditFacilityProposalDialog } from "./credit-facility-proposals/create"

import { CreditFacilityPartialPaymentDialog } from "./credit-facilities/partial-payment"
import { CreateUserDialog } from "./users/create"
import { CreateTermsTemplateDialog } from "./terms-templates/create"
import { CreateCommitteeDialog } from "./committees/create"
import { CreditFacilityDisbursalInitiateDialog } from "./disbursals/create"
import { ExecuteManualTransactionDialog } from "./journal/execute-manual-transaction"
import { CreateCustodianDialog } from "./custodians/create"
import { AddRootNodeDialog } from "./chart-of-accounts/add-root-node-dialog"
import { AddChildNodeDialog } from "./chart-of-accounts/add-child-node-dialog"
import { CreateDepositAccountDialog } from "./deposit-accounts/create"
import { OpenNextYearDialog } from "./fiscal-years/open-next-year"

import {
  CreditFacility,
  Customer,
  CreditFacilityStatus,
  DepositAccountStatus,
  GetWithdrawalDetailsQuery,
  GetPolicyDetailsQuery,
  GetCommitteeDetailsQuery,
  TermsTemplateQuery,
  GetDisbursalDetailsQuery,
  GetDepositAccountDetailsQuery,
  LedgerAccountDetailsFragment,
  FiscalYear,
} from "@/lib/graphql/generated"

export const PATH_CONFIGS = {
  COMMITTEES: "/committees",
  COMMITTEE_DETAILS: /^\/committees\/[^/]+/,

  CREDIT_FACILITY_DETAILS: /^\/credit-facilities\/[^/]+/,

  PROSPECTS: "/prospects",
  CUSTOMER_DETAILS: /^\/customers\/[^/]+/,

  USERS: "/users",
  USER_DETAILS: /^\/users\/[^/]+/,

  TERMS_TEMPLATES: "/terms-templates",
  TERMS_TEMPLATE_DETAILS: /^\/terms-templates\/[^/]+/,

  WITHDRAWAL_DETAILS: /^\/withdrawals\/[^/]+/,
  DEPOSIT_DETAILS: /^\/deposits\/[^/]+/,
  POLICY_DETAILS: /^\/policies\/[^/]+/,
  DISBURSAL_DETAILS: /^\/disbursals\/[^/]+/,

  CUSTODIANS: "/custodians",

  JOURNAL: "/journal",

  ROLES_AND_PERMISSIONS: "/roles-and-permissions",

  FISCAL_YEARS: "/fiscal-years",

  CHART_OF_ACCOUNTS: "/chart-of-accounts",
  LEDGER_ACCOUNTS: "/ledger-accounts",
  LEDGER_TRANSACTIONS: "/ledger-transactions",

  LEDGER_ACCOUNT_DETAILS: /^\/ledger-accounts\/[^/]+/,
  DEPOSIT_ACCOUNT_DETAILS: /^\/deposit-accounts\/[^/]+/,
}

export const ALWAYS_SHOW_DROPDOWN_PATHS: (string | RegExp)[] = [
  PATH_CONFIGS.CHART_OF_ACCOUNTS,
]

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

const isDetailsPage = (currentPath: string) => {
  const segments = currentPath.split("/").filter(Boolean)
  return segments.length >= 2
}

const shouldAlwaysShowDropdown = (currentPath: string) => {
  const isInExceptionList = ALWAYS_SHOW_DROPDOWN_PATHS.some((path) => {
    if (typeof path === "string") {
      return path === currentPath
    } else if (path instanceof RegExp) {
      return path.test(currentPath)
    }
    return false
  })

  return isDetailsPage(currentPath) || isInExceptionList
}

type MenuItem = {
  label: string
  onClick: () => void
  dataTestId: string
  allowedPaths: (string | RegExp)[]
}

const CreateButton = () => {
  const t = useTranslations("CreateButton")
  const router = useRouter()

  const [createProspect, setCreateProspect] = useState(false)
  const [createDeposit, setCreateDeposit] = useState(false)
  const [createWithdrawal, setCreateWithdrawal] = useState(false)
  const [createFacility, setCreateFacility] = useState(false)
  const [initiateDisbursal, setInitiateDisbursal] = useState(false)
  const [makePayment, setMakePayment] = useState(false)
  const [openCreateUserDialog, setOpenCreateUserDialog] = useState(false)
  const [openCreateTermsTemplateDialog, setOpenCreateTermsTemplateDialog] =
    useState(false)
  const [openCreateCommitteeDialog, setOpenCreateCommitteeDialog] = useState(false)
  const [openCreateCustodianDialog, setOpenCreateCustodianDialog] = useState(false)
  const [openExecuteManualTransaction, setOpenExecuteManualTransaction] = useState(false)
  const [openAddAccountDialog, setOpenAddAccountDialog] = useState(false)
  const [openCreateDepositAccountDialog, setOpenCreateDepositAccountDialog] =
    useState(false)
  const [openAddChildAccountDialog, setOpenAddChildAccountDialog] = useState(false)
  const [openOpenNextYearDialog, setOpenOpenNextYearDialog] = useState(false)
  const [showMenu, setShowMenu] = useState(false)

  const {
    customer,
    facility,
    depositAccount,
    ledgerAccount,
    latestFiscalYear,
    setCustomer,
    setDepositAccount,
    setLedgerAccount,
  } = useCreateContext()
  const pathName = usePathname()
  const isFiscalYearsPath = pathName === PATH_CONFIGS.FISCAL_YEARS

  const { nextFiscalYear, canOpenNextFiscalYear } = useMemo(() => {
    if (!latestFiscalYear) return { nextFiscalYear: null, canOpenNextFiscalYear: false }

    const latestFiscalYearYear = getUTCYear(latestFiscalYear.openedAsOf)
    const nextFiscalYear = latestFiscalYearYear !== null ? latestFiscalYearYear + 1 : null
    const nowUtcYear = new Date().getUTCFullYear()

    return {
      nextFiscalYear,
      canOpenNextFiscalYear: nextFiscalYear !== null && nextFiscalYear <= nowUtcYear + 1,
    }
  }, [latestFiscalYear])

  const userIsInCustomerDetailsPage = Boolean(pathName.match(/^\/customers\/.+$/))
  const userIsInDepositAccountDetailsPage = Boolean(
    pathName.match(/^\/deposit-accounts\/.+$/),
  )
  const userIsInLedgerAccountDetailsPage =
    PATH_CONFIGS.LEDGER_ACCOUNT_DETAILS.test(pathName)

  const setCustomerToNullIfNotInCustomerDetails = () => {
    if (!userIsInCustomerDetailsPage) setCustomer(null)
  }

  const setDepositAccountToNullIfNotInDepositAccountDetails = () => {
    if (!userIsInDepositAccountDetailsPage) setDepositAccount(null)
  }

  const setLedgerAccountToNullIfNotInLedgerAccountDetails = () => {
    if (!userIsInLedgerAccountDetailsPage) setLedgerAccount(null)
  }

  const isButtonDisabled = () => {
    if (isFiscalYearsPath) {
      if (!latestFiscalYear) return true
      return !canOpenNextFiscalYear
    }
    if (PATH_CONFIGS.CREDIT_FACILITY_DETAILS.test(pathName)) {
      return !facility || facility.status !== CreditFacilityStatus.Active
    }
    if (PATH_CONFIGS.CUSTOMER_DETAILS.test(pathName)) {
      return customer?.depositAccount?.status === DepositAccountStatus.Closed
    }
    return false
  }

  const getDisabledMessage = () => {
    if (isFiscalYearsPath && isButtonDisabled()) {
      if (!latestFiscalYear) return ""
      return t("disabledMessages.fiscalYearCannotOpenNextYet")
    }
    if (pathName.includes("credit-facilities") && isButtonDisabled()) {
      return t("disabledMessages.creditFacilityMustBeActive")
    }
    if (PATH_CONFIGS.CUSTOMER_DETAILS.test(pathName) && isButtonDisabled()) {
      return t("disabledMessages.depositAccountClosed")
    }
    return ""
  }

  const menuItems: MenuItem[] = [
    {
      label: t("menuItems.deposit"),
      onClick: () => {
        if (!depositAccount) return
        setCreateDeposit(true)
      },
      dataTestId: "create-deposit-button",
      allowedPaths: [PATH_CONFIGS.DEPOSIT_ACCOUNT_DETAILS],
    },
    {
      label: t("menuItems.depositAccount"),
      onClick: () => {
        if (!customer) return
        setOpenCreateDepositAccountDialog(true)
      },
      dataTestId: "create-deposit-account-button",
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("menuItems.withdrawal"),
      onClick: () => {
        if (!depositAccount) return
        setCreateWithdrawal(true)
      },
      dataTestId: "create-withdrawal-button",
      allowedPaths: [PATH_CONFIGS.DEPOSIT_ACCOUNT_DETAILS],
    },
    {
      label: t("menuItems.prospect"),
      onClick: () => setCreateProspect(true),
      dataTestId: "create-prospect-button",
      allowedPaths: [PATH_CONFIGS.PROSPECTS],
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
      allowedPaths: [PATH_CONFIGS.USERS],
    },
    {
      label: t("menuItems.termsTemplate"),
      onClick: () => setOpenCreateTermsTemplateDialog(true),
      dataTestId: "create-terms-template-button",
      allowedPaths: [PATH_CONFIGS.TERMS_TEMPLATES],
    },
    {
      label: t("menuItems.committee"),
      onClick: () => setOpenCreateCommitteeDialog(true),
      dataTestId: "create-committee-button",
      allowedPaths: [PATH_CONFIGS.COMMITTEES],
    },
    {
      label: t("menuItems.nextFiscalYear"),
      onClick: () => {
        if (!canOpenNextFiscalYear || !latestFiscalYear || nextFiscalYear === null) return
        setOpenOpenNextYearDialog(true)
      },
      dataTestId: "create-next-fiscal-year-button",
      allowedPaths: [PATH_CONFIGS.FISCAL_YEARS],
    },
    {
      label: t("menuItems.custodian"),
      onClick: () => setOpenCreateCustodianDialog(true),
      dataTestId: "create-custodian-button",
      allowedPaths: [PATH_CONFIGS.CUSTODIANS],
    },
    {
      label: t("menuItems.executeManualTransaction"),
      onClick: () => setOpenExecuteManualTransaction(true),
      dataTestId: "execute-manual-transaction-button",
      allowedPaths: [PATH_CONFIGS.LEDGER_TRANSACTIONS],
    },
    {
      label: t("menuItems.role"),
      onClick: () => router.push("/roles-and-permissions/create"),
      dataTestId: "create-role-button",
      allowedPaths: [PATH_CONFIGS.ROLES_AND_PERMISSIONS],
    },
    {
      label: t("menuItems.account"),
      onClick: () => setOpenAddAccountDialog(true),
      dataTestId: "create-account-button",
      allowedPaths: [PATH_CONFIGS.CHART_OF_ACCOUNTS, PATH_CONFIGS.LEDGER_ACCOUNTS],
    },
    {
      label: t("menuItems.subAccount"),
      onClick: () => {
        if (!ledgerAccount) return
        setOpenAddChildAccountDialog(true)
      },
      dataTestId: "add-sub-account-button",
      allowedPaths: [PATH_CONFIGS.LEDGER_ACCOUNT_DETAILS],
    },
  ]

  const getAvailableMenuItems = () => {
    return menuItems.filter((item) => {
      const isPathAllowed = isItemAllowedOnCurrentPath(item.allowedPaths, pathName)

      // TODO: add ability to disable options instead of hiding them
      // Hide deposit and withdrawal options if deposit account is not active
      if (
        item.label === t("menuItems.deposit") ||
        item.label === t("menuItems.withdrawal")
      ) {
        // Only on deposit account details page, check depositAccount directly
        return isPathAllowed && depositAccount?.status === DepositAccountStatus.Active
      }

      // Hide deposit account creation if account already exists
      if (item.label === t("menuItems.depositAccount")) {
        return isPathAllowed && !customer?.depositAccount
      }

      // Show credit facility proposal only if customer has a deposit account
      if (item.label === t("menuItems.creditFacility")) {
        return isPathAllowed && Boolean(customer?.depositAccount)
      }

      // Hide disbursal option if facility is single disbursal and already has disbursals
      if (item.label === t("menuItems.disbursal")) {
        if (
          facility?.creditFacilityTerms?.disbursalPolicy === "SINGLE_DISBURSAL" &&
          facility?.disbursals &&
          facility.disbursals.length > 0
        ) {
          return false
        }
      }
      return isPathAllowed
    })
  }

  const decideCreation = () => {
    setShowMenu(false)
    const availableItems = getAvailableMenuItems()

    const forceDropdown = shouldAlwaysShowDropdown(pathName)
    if (availableItems.length === 1 && !forceDropdown) {
      availableItems[0].onClick()
      return
    }

    if (availableItems.length > 0) {
      setShowMenu(true)
    }
  }

  const availableItems = getAvailableMenuItems()
  const showCreate = showCreateButton(pathName) && availableItems.length > 0
  const disabled = isButtonDisabled()
  const message = getDisabledMessage()

  return (
    <>
      {showCreate ? (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <div>
                <DropdownMenu
                  open={showMenu && !disabled}
                  onOpenChange={(open: boolean) => {
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
      ) : null}

      <CreateProspectDialog
        setOpenCreateProspectDialog={setCreateProspect}
        openCreateProspectDialog={createProspect}
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

      <CreateCustodianDialog
        openCreateCustodianDialog={openCreateCustodianDialog}
        setOpenCreateCustodianDialog={setOpenCreateCustodianDialog}
      />

      <ExecuteManualTransactionDialog
        openExecuteManualTransaction={openExecuteManualTransaction}
        setOpenExecuteManualTransaction={setOpenExecuteManualTransaction}
      />

      <AddRootNodeDialog
        open={openAddAccountDialog}
        onOpenChange={setOpenAddAccountDialog}
      />

      {customer && !customer.depositAccount && (
        <CreateDepositAccountDialog
          openCreateDepositAccountDialog={openCreateDepositAccountDialog}
          setOpenCreateDepositAccountDialog={() => {
            setCustomerToNullIfNotInCustomerDetails()
            setOpenCreateDepositAccountDialog(false)
          }}
          customerId={customer.customerId}
        />
      )}

      {depositAccount && (
        <>
          <CreateDepositDialog
            openCreateDepositDialog={createDeposit}
            setOpenCreateDepositDialog={() => {
              setDepositAccountToNullIfNotInDepositAccountDetails()
              setCreateDeposit(false)
            }}
            depositAccountId={depositAccount.depositAccountId}
          />

          <WithdrawalInitiateDialog
            openWithdrawalInitiateDialog={createWithdrawal}
            setOpenWithdrawalInitiateDialog={() => {
              setDepositAccountToNullIfNotInDepositAccountDetails()
              setCreateWithdrawal(false)
            }}
            depositAccountId={depositAccount.depositAccountId}
          />
        </>
      )}

      {customer && customer.depositAccount && (
        <CreateCreditFacilityProposalDialog
          openCreateCreditFacilityProposalDialog={createFacility}
          setOpenCreateCreditFacilityProposalDialog={() => {
            setCustomerToNullIfNotInCustomerDetails()
            setCreateFacility(false)
          }}
          customerId={customer.customerId}
        />
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
            userCanRecordPaymentWithDate={facility.userCanRecordPaymentWithDate}
            openDialog={makePayment}
            setOpenDialog={() => {
              setMakePayment(false)
            }}
          />
        </>
      )}

      {ledgerAccount?.code && (
        <AddChildNodeDialog
          open={openAddChildAccountDialog}
          onOpenChange={(open: boolean) => {
            if (!open) {
              setLedgerAccountToNullIfNotInLedgerAccountDetails()
            }
            setOpenAddChildAccountDialog(open)
          }}
          parentCode={ledgerAccount.code}
          parentName={ledgerAccount.name}
        />
      )}

      {canOpenNextFiscalYear && latestFiscalYear && nextFiscalYear !== null && (
        <OpenNextYearDialog
          fiscalYear={latestFiscalYear}
          nextFiscalYear={nextFiscalYear}
          open={openOpenNextYearDialog}
          onOpenChange={setOpenOpenNextYearDialog}
        />
      )}
    </>
  )
}

type ICustomer = Customer | null
type IFacility = CreditFacility | null
type ITermsTemplate = TermsTemplateQuery["termsTemplate"] | null
type IWithdraw = GetWithdrawalDetailsQuery["withdrawalByPublicId"] | null
type IPolicy = GetPolicyDetailsQuery["policy"] | null
type ICommittee = GetCommitteeDetailsQuery["committee"] | null
type IDisbursal = GetDisbursalDetailsQuery["disbursalByPublicId"] | null
type IDepositAccount = NonNullable<
  GetDepositAccountDetailsQuery["depositAccountByPublicId"]
> | null
type ILedgerAccount = LedgerAccountDetailsFragment | null
type ILatestFiscalYear = Pick<FiscalYear, "fiscalYearId" | "openedAsOf"> | null

type CreateContext = {
  customer: ICustomer
  facility: IFacility
  termsTemplate: ITermsTemplate
  withdraw: IWithdraw
  policy: IPolicy
  committee: ICommittee
  disbursal: IDisbursal
  depositAccount: IDepositAccount
  ledgerAccount: ILedgerAccount
  latestFiscalYear: ILatestFiscalYear

  setCustomer: React.Dispatch<React.SetStateAction<ICustomer>>
  setFacility: React.Dispatch<React.SetStateAction<IFacility>>
  setTermsTemplate: React.Dispatch<React.SetStateAction<ITermsTemplate>>
  setWithdraw: React.Dispatch<React.SetStateAction<IWithdraw>>
  setPolicy: React.Dispatch<React.SetStateAction<IPolicy>>
  setCommittee: React.Dispatch<React.SetStateAction<ICommittee>>
  setDisbursal: React.Dispatch<React.SetStateAction<IDisbursal>>
  setDepositAccount: React.Dispatch<React.SetStateAction<IDepositAccount>>
  setLedgerAccount: React.Dispatch<React.SetStateAction<ILedgerAccount>>
  setLatestFiscalYear: React.Dispatch<React.SetStateAction<ILatestFiscalYear>>
}

const CreateContext = createContext<CreateContext>({
  customer: null,
  facility: null,
  termsTemplate: null,
  withdraw: null,
  policy: null,
  committee: null,
  disbursal: null,
  depositAccount: null,
  ledgerAccount: null,
  latestFiscalYear: null,

  setCustomer: () => {},
  setFacility: () => {},
  setTermsTemplate: () => {},
  setWithdraw: () => {},
  setPolicy: () => {},
  setCommittee: () => {},
  setDisbursal: () => {},
  setDepositAccount: () => {},
  setLedgerAccount: () => {},
  setLatestFiscalYear: () => {},
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
  const [depositAccount, setDepositAccount] = useState<IDepositAccount>(null)
  const [ledgerAccount, setLedgerAccount] = useState<ILedgerAccount>(null)
  const [latestFiscalYear, setLatestFiscalYear] = useState<ILatestFiscalYear>(null)

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
        depositAccount,
        ledgerAccount,
        latestFiscalYear,

        setCustomer,
        setFacility,
        setTermsTemplate,
        setWithdraw,
        setPolicy,
        setCommittee,
        setDisbursal,
        setDepositAccount,
        setLedgerAccount,
        setLatestFiscalYear,
      }}
    >
      {children}
    </CreateContext.Provider>
  )
}

export const useCreateContext = () => useContext(CreateContext)

export default CreateButton
