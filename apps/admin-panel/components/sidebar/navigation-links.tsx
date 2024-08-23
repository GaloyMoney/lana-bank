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

import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/primitive/collapsible"

const navLinks = [
  { href: "/customers", label: "Customers", icon: IoPersonOutline },
  { href: "/loans", label: "Loan", icon: IoReceiptOutline },
  { href: "/terms", label: "Default Terms", icon: IoDocumentOutline },
  { href: "/audit", label: "Audit Logs", icon: IoTimeOutline },
  { href: "/users", label: "Users", icon: RiAdminLine },
  { href: "/deposits", label: "Deposits", icon: PiHandDeposit },
  { href: "/withdrawals", label: "Withdrawals", icon: PiHandWithdraw },

  {
    label: "Financials",
    icon: IoCashOutline,
    subMenu: [
      { href: "/chart-of-accounts", label: "Chart of Accounts" },
      { href: "/balance-sheet", label: "Balance Sheet" },
      { href: "/profit-and-loss", label: "Profit and Loss" },
      { href: "/trial-balance", label: "Trial Balance" },
    ],
  },
]

const NavigationLinks = () => {
  const pathname = usePathname()
  const [openSubmenu, setOpenSubmenu] = useState<string | null>(null)

  const toggleSubmenu = (label: string) => {
    setOpenSubmenu(openSubmenu === label ? null : label)
  }

  return (
    <nav className="flex flex-col gap-4 text-textColor-secondary pr-4">
      {navLinks.map((link, index) => (
        <React.Fragment key={index}>
          {link.subMenu ? (
            <Collapsible
              open={openSubmenu === link.label}
              onOpenChange={() => toggleSubmenu(link.label)}
            >
              <CollapsibleTrigger className="flex items-center justify-between w-full hover:text-textColor-primary pr-2">
                <div className="flex items-center gap-4">
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
                      pathname === subItem.href && "text-primary"
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
              className={`hover:text-textColor-primary ${pathname === link.href && "text-primary"}`}
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
