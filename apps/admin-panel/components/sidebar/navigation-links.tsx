"use client"
import React, { useState } from "react"
import {
  IoChevronDown,
  IoChevronUp,
  IoPersonOutline,
  IoReceiptOutline,
  IoCashOutline,
  IoDocumentOutline,
  IoTimeOutline,
} from "react-icons/io5"
import Link from "next/link"
import { usePathname } from "next/navigation"

import { RiAdminLine } from "react-icons/ri"

import { PiHandDeposit, PiHandWithdraw } from "react-icons/pi"
import { HiOutlineDocumentReport } from "react-icons/hi"
import { MdOutlinePolicy } from "react-icons/md"

import { gql } from "@apollo/client"

import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/primitive/collapsible"
import { useMeQuery } from "@/lib/graphql/generated"

gql`
  query Me {
    me {
      user {
        userId
        email
        roles
      }
      subjectCanCreateUser
      subjectCanCreateCustomer
      subjectCanCreateTermsTemplate
      visibleNavigationItems {
        term
        user
        customer
        deposit
        withdraw
        audit
        financials
        creditFacilities
        governance {
          committee
          policy
          approvalProcess
        }
      }
    }
  }
`

const NavigationLinks = () => {
  const { data, loading, error } = useMeQuery()
  const pathname = usePathname()
  const [openSubmenu, setOpenSubmenu] = useState<string | null>(null)
  const toggleSubmenu = (label: string) => {
    setOpenSubmenu(openSubmenu === label ? null : label)
  }

  if (loading) return null
  if (error) return <div>Error loading navigation</div>

  const visibleItems = data?.me?.visibleNavigationItems

  const navLinks = [
    {
      href: "/customers",
      label: "Customers",
      icon: IoPersonOutline,
      visible: visibleItems?.customer,
    },
    {
      href: "/credit-facilities",
      label: "Credit Facilities",
      icon: IoReceiptOutline,
      visible: visibleItems?.creditFacilities,
    },
    {
      href: "/terms-templates",
      label: "Terms Templates",
      icon: IoDocumentOutline,
      visible: visibleItems?.term,
    },
    {
      href: "/audit",
      label: "Audit Logs",
      icon: IoTimeOutline,
      visible: visibleItems?.audit,
    },
    { href: "/users", label: "Users", icon: RiAdminLine, visible: visibleItems?.user },
    {
      href: "/deposits",
      label: "Deposits",
      icon: PiHandDeposit,
      visible: visibleItems?.deposit,
    },
    {
      href: "/withdrawals",
      label: "Withdrawals",
      icon: PiHandWithdraw,
      visible: visibleItems?.withdraw,
    },
    {
      href: "/regulatory-reporting",
      label: "Regulatory Reporting",
      icon: HiOutlineDocumentReport,
      visible: true,
    },
    {
      label: "Governance",
      icon: MdOutlinePolicy,
      visible:
        visibleItems?.governance.committee ||
        visibleItems?.governance.policy ||
        visibleItems?.governance.approvalProcess,
      subMenu: [
        { href: "/committees", label: "Committees" },
        { href: "/policies", label: "Policies" },
      ],
    },
    {
      label: "Financials",
      icon: IoCashOutline,
      visible: visibleItems?.financials,
      subMenu: [
        { href: "/chart-of-accounts", label: "Chart of Accounts" },
        { href: "/balance-sheet", label: "Balance Sheet" },
        { href: "/profit-and-loss", label: "Profit and Loss" },
        { href: "/trial-balance", label: "Trial Balance" },
      ],
    },
  ]

  const isActiveLink = (href: string) => {
    if (pathname === href) return true
    if (href !== "/" && pathname.startsWith(href)) return true
    return false
  }

  return (
    <nav className="flex flex-col gap-4 text-textColor-secondary pr-4">
      {navLinks
        .filter((link) => link.visible)
        .map((link, index) => (
          <React.Fragment key={index}>
            {link.subMenu ? (
              <Collapsible
                open={openSubmenu === link.label}
                onOpenChange={() => toggleSubmenu(link.label)}
              >
                <CollapsibleTrigger className="flex items-center justify-between w-full hover:text-textColor-primary pr-2">
                  <div
                    className={`flex items-center gap-4 ${link.subMenu.some((subItem) => isActiveLink(subItem.href)) ? "text-primary" : ""}`}
                  >
                    <link.icon className="w-4 h-4" />
                    {link.label}
                  </div>
                  {openSubmenu === link.label ? (
                    <IoChevronUp className="w-4 h-4" />
                  ) : (
                    <IoChevronDown className="w-4 h-4" />
                  )}
                </CollapsibleTrigger>
                <CollapsibleContent className="ml-6 mt-2">
                  {link.subMenu.map((subItem, subIndex) => (
                    <Link
                      key={subIndex}
                      href={subItem.href}
                      prefetch={false}
                      className={`block p-1.5 px-2 hover:text-textColor-primary ${
                        isActiveLink(subItem.href) ? "text-primary" : ""
                      }`}
                    >
                      <div className="flex items-center gap-4">{subItem.label}</div>
                    </Link>
                  ))}
                </CollapsibleContent>
              </Collapsible>
            ) : (
              <Link
                href={link.href}
                prefetch={false}
                className={`hover:text-textColor-primary ${
                  isActiveLink(link.href) ? "text-primary" : ""
                }`}
              >
                <div className="flex items-center gap-4 rounded-md">
                  <link.icon className="w-4 h-4" />
                  {link.label}
                </div>
              </Link>
            )}
          </React.Fragment>
        ))}
    </nav>
  )
}

export { NavigationLinks }
