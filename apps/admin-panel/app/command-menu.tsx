"use client"

import React, { useEffect, useState } from "react"
import { usePathname, useRouter } from "next/navigation"
import { useTranslations } from "next-intl"

import {
  CheckCircle2,
  CheckSquare,
  FileEdit,
  Keyboard,
  Plus,
  Settings,
  Shield,
  Wallet,
  XCircle,
  XSquare,
} from "lucide-react"

import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@lana/web/ui/command"

import { CreateCustomerDialog } from "./customers/create"
import { CreateDepositDialog } from "./deposits/create"
import { WithdrawalInitiateDialog } from "./withdrawals/initiate"
import { CreateCreditFacilityDialog } from "./credit-facilities/create"
import { CreditFacilityPartialPaymentDialog } from "./credit-facilities/partial-payment"
import { CreditFacilityCollateralUpdateDialog } from "./credit-facilities/collateral-update"
import { CreateUserDialog } from "./users/create"
import { CreateTermsTemplateDialog } from "./terms-templates/create"
import { CreateCommitteeDialog } from "./committees/create"
import { CreditFacilityDisbursalInitiateDialog } from "./disbursals/create"
import ApprovalDialog from "./actions/approve"
import DenialDialog from "./actions/deny"

import { PATH_CONFIGS, useCreateContext } from "./create"

import { UpdateTermsTemplateDialog } from "./terms-templates/[terms-template-id]/update"

import { WithdrawalConfirmDialog } from "./withdrawals/[withdrawal-id]/confirm"
import { WithdrawalCancelDialog } from "./withdrawals/[withdrawal-id]/cancel"
import CommitteeAssignmentDialog from "./policies/[policy-id]/assign-to-committee"
import AddUserCommitteeDialog from "./committees/add-user"

import {
  ApprovalProcessStatus,
  CreditFacilityStatus,
  WithdrawalStatus,
} from "@/lib/graphql/generated"

import { useNavItems } from "@/components/app-sidebar/nav-items"

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

type ApprovalAction = {
  type: "facility" | "withdraw" | "disbursal" | null
  action: "approve" | "deny" | null
}

type groups = "main" | "navigation" | "actions"

const CommandMenu = () => {
  const router = useRouter()
  const pathName = usePathname()

  const t = useTranslations("CommandMenu")

  const {
    navDashboardItems,
    navLoansItems,
    navTransactionItems,
    navAdminItems,
    navFinanceItems,
  } = useNavItems()

  // Combine all nav items
  const allNavItems = [
    ...navDashboardItems,
    ...navLoansItems,
    ...navTransactionItems,
    ...navAdminItems,
    ...navFinanceItems,
  ]

  const [open, setOpen] = useState(false)
  const [pages, setPages] = useState<groups>("main")

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
  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] = useState(false)
  const [openUpdateTermsTemplateDialog, setOpenUpdateTermsTemplateDialog] =
    useState(false)
  const [openWithdrawalConfirmDialog, setOpenWithdrawalConfirmDialog] = useState(false)
  const [openWithdrawalCancelDialog, setOpenWithdrawalCancelDialog] = useState(false)
  const [openPolicyAssignDialog, setOpenPolicyAssignDialog] = useState(false)
  const [openAddUserCommitteeDialog, setOpenAddUserCommitteeDialog] = useState(false)

  const [approvalAction, setApprovalAction] = useState<ApprovalAction>({
    type: null,
    action: null,
  })

  const getActiveEntity = () => {
    if (facility) return facility
    if (withdraw) return withdraw
    if (disbursal) return disbursal
    return null
  }

  const getActiveEntityType = (): "facility" | "withdraw" | "disbursal" | null => {
    if (facility) return "facility"
    if (withdraw) return "withdraw"
    if (disbursal) return "disbursal"
    return null
  }

  const { customer, facility, termsTemplate, withdraw, policy, committee, disbursal } =
    useCreateContext()

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        setPages("main")
        setOpen((open) => !open)
      }
      if (e.shiftKey && e.key === "N") {
        const activeElement = document.activeElement?.tagName?.toLowerCase()
        const ignoredElements = ["input", "textarea", "select"]
        if (activeElement && !ignoredElements.includes(activeElement)) {
          e.preventDefault()
          setOpen((open) => !open)
          setPages("navigation")
        }
      }
      if (e.shiftKey && e.key === "A") {
        const activeElement = document.activeElement?.tagName?.toLowerCase()
        const ignoredElements = ["input", "textarea", "select"]
        if (activeElement && !ignoredElements.includes(activeElement)) {
          e.preventDefault()
          setOpen((open) => !open)
          setPages("actions")
        }
      }
    }
    document.addEventListener("keydown", down)
    return () => document.removeEventListener("keydown", down)
  }, [])

  const menuItems = [
    {
      label: t("actions.createDeposit"),
      icon: Plus,
      action: () => {
        if (!customer) return
        setCreateDeposit(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("actions.createWithdrawal"),
      icon: Plus,
      action: () => {
        if (!customer) return
        setCreateWithdrawal(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("actions.createCustomer"),
      icon: Plus,
      action: () => {
        setCreateCustomer(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CUSTOMERS, PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("actions.createCreditFacility"),
      icon: Plus,
      action: () => {
        if (!customer) return
        setCreateFacility(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CUSTOMER_DETAILS],
    },
    {
      label: t("actions.updateCollateral"),
      icon: Shield,
      action: () => {
        if (!facility) return
        setOpenCollateralUpdateDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CREDIT_FACILITY_DETAILS],
      condition: () =>
        facility?.subjectCanUpdateCollateral &&
        facility?.status !== CreditFacilityStatus.Closed &&
        facility?.status !== CreditFacilityStatus.Matured,
    },
    {
      label: t("actions.createDisbursal"),
      icon: Plus,
      action: () => {
        if (!facility) return
        setInitiateDisbursal(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CREDIT_FACILITY_DETAILS],
      condition: () => facility?.status === CreditFacilityStatus.Active,
    },
    {
      label: t("actions.makePayment"),
      icon: Wallet,
      action: () => {
        if (!facility) return
        setMakePayment(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.CREDIT_FACILITY_DETAILS],
      condition: () => facility?.status === CreditFacilityStatus.Active,
    },
    {
      label: t("actions.createUser"),
      icon: Plus,
      action: () => {
        setOpenCreateUserDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.USERS, PATH_CONFIGS.USER_DETAILS],
    },
    {
      label: t("actions.updateTermsTemplate"),
      icon: FileEdit,
      action: () => {
        if (!termsTemplate) return
        setOpenUpdateTermsTemplateDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.TERMS_TEMPLATE_DETAILS],
      condition: () => termsTemplate?.subjectCanUpdateTermsTemplate,
    },
    {
      label: t("actions.createTermsTemplate"),
      icon: Plus,
      action: () => {
        setOpenCreateTermsTemplateDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.TERMS_TEMPLATES, PATH_CONFIGS.TERMS_TEMPLATE_DETAILS],
    },
    {
      label: t("actions.createCommittee"),
      icon: Plus,
      action: () => {
        setOpenCreateCommitteeDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.COMMITTEES, PATH_CONFIGS.COMMITTEE_DETAILS],
    },
    {
      label: t("actions.confirmWithdrawal"),
      icon: CheckCircle2,
      action: () => {
        if (!withdraw) return
        setOpenWithdrawalConfirmDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.WITHDRAWAL_DETAILS],
      condition: () => withdraw?.status === WithdrawalStatus.PendingConfirmation,
    },
    {
      label: t("actions.cancelWithdrawal"),
      icon: XCircle,
      action: () => {
        if (!withdraw) return
        setOpenWithdrawalCancelDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.WITHDRAWAL_DETAILS],
      condition: () => withdraw?.status === WithdrawalStatus.PendingConfirmation,
    },
    {
      label: t("actions.assignCommittee"),
      icon: Settings,
      action: () => {
        if (!policy) return
        setOpenPolicyAssignDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.POLICY_DETAILS],
    },
    {
      label: t("actions.addCommitteeMember"),
      icon: Plus,
      action: () => {
        if (!committee) return
        setOpenAddUserCommitteeDialog(true)
        setOpen(false)
      },
      allowedPaths: [PATH_CONFIGS.COMMITTEE_DETAILS],
    },
    {
      label: t("actions.approve"),
      icon: CheckSquare,
      action: () => {
        setApprovalAction({ type: getActiveEntityType(), action: "approve" })
        setOpen(false)
      },
      allowedPaths: [
        PATH_CONFIGS.CREDIT_FACILITY_DETAILS,
        PATH_CONFIGS.WITHDRAWAL_DETAILS,
        PATH_CONFIGS.DISBURSAL_DETAILS,
      ],
      condition: () => {
        const entity = getActiveEntity()
        return (
          entity?.approvalProcess?.status === ApprovalProcessStatus.InProgress &&
          entity?.approvalProcess?.subjectCanSubmitDecision
        )
      },
    },
    {
      label: t("actions.deny"),
      icon: XSquare,
      action: () => {
        setApprovalAction({ type: getActiveEntityType(), action: "deny" })
        setOpen(false)
      },
      allowedPaths: [
        PATH_CONFIGS.CREDIT_FACILITY_DETAILS,
        PATH_CONFIGS.WITHDRAWAL_DETAILS,
        PATH_CONFIGS.DISBURSAL_DETAILS,
      ],
      condition: () => {
        const entity = getActiveEntity()
        return (
          entity?.approvalProcess?.status === ApprovalProcessStatus.InProgress &&
          entity?.approvalProcess?.subjectCanSubmitDecision
        )
      },
    },
  ]

  const availableItems = menuItems.filter((item) =>
    isItemAllowedOnCurrentPath(item.allowedPaths, pathName),
  )

  return (
    <>
      <CommandDialog open={open} onOpenChange={setOpen}>
        <Command className="rounded-lg border shadow-md">
          <CommandInput
            placeholder={
              pages === "navigation"
                ? t("placeholders.searchNavigation")
                : pages === "actions"
                  ? t("placeholders.searchActions")
                  : t("placeholders.whatDoYouNeed")
            }
          />
          <CommandList>
            <CommandEmpty>{t("noResults")}</CommandEmpty>
            {pages === "main" ? (
              <>
                {availableItems.length > 0 && (
                  <>
                    <CommandSeparator />
                    <CommandGroup
                      heading={
                        <KeyboardControlHeading
                          heading={t("headings.availableActions")}
                          combination="Shift + A"
                        />
                      }
                    >
                      {availableItems.map((item) => (
                        <CommandItem
                          key={item.label}
                          disabled={item.condition && !item.condition()}
                          onSelect={() => {
                            item.action()
                          }}
                        >
                          <item.icon className="mr-2 h-4 w-4" />
                          {item.label}
                        </CommandItem>
                      ))}
                    </CommandGroup>
                  </>
                )}
                <CommandSeparator />
                <CommandGroup
                  heading={
                    <KeyboardControlHeading
                      heading={t("headings.navigation")}
                      combination="Shift + N"
                    />
                  }
                >
                  {allNavItems.map((item) => (
                    <CommandItem
                      key={item.url}
                      onSelect={() => {
                        router.push(item.url)
                        setOpen(false)
                      }}
                      className="flex items-center gap-2"
                    >
                      <item.icon className="h-4 w-4" />
                      <span>{item.title}</span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </>
            ) : pages === "actions" ? (
              <CommandGroup heading={t("headings.availableActions")}>
                {availableItems.map((item) => (
                  <CommandItem
                    key={item.label}
                    disabled={item.condition && !item.condition()}
                    onSelect={() => {
                      item.action()
                    }}
                  >
                    <item.icon className="mr-2 h-4 w-4" />
                    {item.label}
                  </CommandItem>
                ))}
              </CommandGroup>
            ) : (
              <CommandGroup heading={t("headings.navigation")}>
                {allNavItems.map((item) => (
                  <CommandItem
                    key={item.url}
                    onSelect={() => {
                      setOpen(false)
                      router.push(item.url)
                    }}
                    className="flex items-center gap-2"
                  >
                    <item.icon className="h-4 w-4" />
                    <span>{item.title}</span>
                  </CommandItem>
                ))}
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </CommandDialog>

      <CreateCustomerDialog
        openCreateCustomerDialog={createCustomer}
        setOpenCreateCustomerDialog={setCreateCustomer}
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
            setOpenCreateDepositDialog={() => setCreateDeposit(false)}
            depositAccountId={customer.depositAccount.depositAccountId}
          />
          <WithdrawalInitiateDialog
            openWithdrawalInitiateDialog={createWithdrawal}
            setOpenWithdrawalInitiateDialog={() => setCreateWithdrawal(false)}
            depositAccountId={customer.depositAccount.depositAccountId}
          />
          <CreateCreditFacilityDialog
            openCreateCreditFacilityDialog={createFacility}
            setOpenCreateCreditFacilityDialog={() => setCreateFacility(false)}
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
          <CreditFacilityCollateralUpdateDialog
            creditFacilityId={facility.creditFacilityId}
            openDialog={openCollateralUpdateDialog}
            setOpenDialog={setOpenCollateralUpdateDialog}
          />
        </>
      )}

      {termsTemplate && (
        <UpdateTermsTemplateDialog
          termsTemplate={termsTemplate}
          openUpdateTermsTemplateDialog={openUpdateTermsTemplateDialog}
          setOpenUpdateTermsTemplateDialog={() => setOpenUpdateTermsTemplateDialog(false)}
        />
      )}

      {withdraw && (
        <>
          <WithdrawalConfirmDialog
            withdrawalData={withdraw}
            openWithdrawalConfirmDialog={openWithdrawalConfirmDialog}
            setOpenWithdrawalConfirmDialog={() => setOpenWithdrawalConfirmDialog(false)}
          />
          <WithdrawalCancelDialog
            withdrawalData={withdraw}
            openWithdrawalCancelDialog={openWithdrawalCancelDialog}
            setOpenWithdrawalCancelDialog={() => setOpenWithdrawalCancelDialog(false)}
          />
        </>
      )}

      {policy && (
        <CommitteeAssignmentDialog
          policyId={policy.policyId}
          openAssignDialog={openPolicyAssignDialog}
          setOpenAssignDialog={setOpenPolicyAssignDialog}
        />
      )}

      {committee && (
        <AddUserCommitteeDialog
          committeeId={committee.committeeId}
          openAddUserDialog={openAddUserCommitteeDialog}
          setOpenAddUserDialog={setOpenAddUserCommitteeDialog}
        />
      )}

      {approvalAction.type &&
        (() => {
          const currentApprovalProcess =
            approvalAction.type === "facility"
              ? facility?.approvalProcess
              : approvalAction.type === "withdraw"
                ? withdraw?.approvalProcess
                : approvalAction.type === "disbursal"
                  ? disbursal?.approvalProcess
                  : null

          return currentApprovalProcess ? (
            <>
              <ApprovalDialog
                approvalProcess={currentApprovalProcess}
                openApprovalDialog={approvalAction.action === "approve"}
                setOpenApprovalDialog={() =>
                  setApprovalAction({ type: null, action: null })
                }
              />
              <DenialDialog
                approvalProcess={currentApprovalProcess}
                openDenialDialog={approvalAction.action === "deny"}
                setOpenDenialDialog={() =>
                  setApprovalAction({ type: null, action: null })
                }
              />
            </>
          ) : null
        })()}
      <KeyboardInstructions />
    </>
  )
}

export { CommandMenu }

function KeyboardControlHeading({
  heading,
  combination,
}: {
  heading: string
  combination: string
}) {
  return (
    <div className="flex items-center justify-between">
      <span>{heading}</span>
      <kbd className="ml-auto pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
        <span className="text-xs">{combination}</span>
      </kbd>
    </div>
  )
}

const KeyboardInstructions = () => {
  const t = useTranslations("CommandMenu")
  const [isMac, setIsMac] = useState(false)
  useEffect(() => {
    const macPlatforms = ["Macintosh", "MacIntel", "MacPPC", "Mac68K"]
    const userAgent = window.navigator.userAgent
    const platform = window.navigator.platform

    setIsMac(
      macPlatforms.includes(platform) ||
        userAgent.includes("Mac") ||
        /Mac/.test(navigator.platform),
    )
  }, [])

  return (
    <div className="fixed bottom-4 right-4 hidden md:flex items-center gap-2 rounded-lg bg-secondary/80 px-3 py-2 text-sm text-secondary-foreground backdrop-blur z-10">
      <Keyboard className="h-4 w-4" />
      <span>{t("keyboardInstructions.commandPalette")}</span>
      <kbd className="rounded border bg-background px-1.5 text-xs font-semibold">
        {isMac ? "cmd" : "Ctrl"}
      </kbd>
      <span>+</span>
      <kbd className="rounded border bg-background px-1.5 text-xs font-semibold">K</kbd>
    </div>
  )
}
