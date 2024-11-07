"use client"

import { useState } from "react"
import { usePathname } from "next/navigation"
import Link from "next/link"
import { motion, AnimatePresence } from "framer-motion"
import classNames from "classnames"

import {
  HiCursorClick,
  HiUser,
  HiHome,
  HiViewGrid,
  HiUserGroup,
  HiArrowCircleDown,
  HiArrowCircleUp,
  HiDocumentReport,
  HiGlobe,
  HiGlobeAlt,
  HiCurrencyDollar,
  HiCash,
  HiLightningBolt,
  HiTemplate,
  HiViewBoards,
  HiOutlineMenu,
  HiLockClosed,
} from "react-icons/hi"

import Avatar from "./avatar"

import { Logo } from "@/components/new"
import { useGetRealtimePriceUpdatesQuery } from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"
import { Skeleton } from "@/components/primitive/skeleton"

const NavBar = () => {
  const [isOpen, setIsOpen] = useState(false)

  const { data, loading } = useGetRealtimePriceUpdatesQuery()

  const usdBtcRate = currencyConverter
    .centsToUsd(data?.realtimePrice?.usdCentsPerBtc)
    .toLocaleString()

  const NavItems = () => (
    <nav className="m-4">
      <ul className="flex flex-col space-y-[1px]">
        <MenuItem icon={HiHome} title="Dashboard" to="/dashboard" />
        <MenuItem icon={HiCursorClick} title="Actions" to="/actions" />
        <div className="h-4"></div>
        <MenuItem icon={HiUser} title="Customers" to="/customers" />
        <MenuItem icon={HiViewGrid} title="Credit Facilities" to="/credit-facilities" />
        <MenuItem icon={HiViewBoards} title="Disbursals" to="/disbursals" />
        <MenuItem icon={HiTemplate} title="Terms Templates" to="/terms-templates" />
        <MenuItem icon={HiUserGroup} title="Users" to="/users" />
        <MenuItem icon={HiGlobe} title="Committees" to="/committees" />
        <MenuItem icon={HiLockClosed} title="Policies" to="/policies" />
        <MenuItem icon={HiArrowCircleDown} title="Deposits" to="/deposits" />
        <MenuItem icon={HiArrowCircleUp} title="Withdrawals" to="/withdrawals" />
        <MenuItem
          icon={HiDocumentReport}
          title="Regulatory Reporting"
          to="/regulatory-reporting"
        />
        <div className="h-4"></div>
        <MenuItem icon={HiGlobeAlt} title="Chart of Accounts" to="/chart-of-accounts" />
        <MenuItem icon={HiCurrencyDollar} title="Balance Sheet" to="/balance-sheet" />
        <MenuItem icon={HiCash} title="Profit & Loss" to="/profit-and-loss" />
        <MenuItem icon={HiLightningBolt} title="Trial Balance" to="/trial-balance" />
      </ul>
    </nav>
  )

  return (
    <div className="md:w-64">
      {/* Mobile Top Bar */}
      <div className="fixed top-0 left-0 right-0 z-50 border-b bg-background md:hidden">
        <div className="flex items-center justify-between px-4 py-2">
          <button onClick={() => setIsOpen(true)} aria-label="Toggle navigation menu">
            <HiOutlineMenu className="w-8 h-8 text-foreground" />
          </button>
          <Logo width={30} />
          <Avatar />
        </div>
      </div>

      {/* Desktop Sidebar */}
      <div className="hidden md:flex md:flex-col min-h-screen justify-between bg-background">
        <div>
          <div className="flex justify-between items-center w-full p-5">
            <Logo width={30} />
            <Avatar />
          </div>
          <NavItems />
        </div>
        {loading ? (
          <Skeleton className="h-4 mb-5 ml-4 w-48" />
        ) : (
          <div className="p-5 text-sm text-muted-foreground">
            USD/BTC Market Rate:{" "}
            {String(usdBtcRate) === "NaN" ? "Not Available" : `$${usdBtcRate}`}
          </div>
        )}
      </div>

      {/* Mobile Sidebar */}
      <AnimatePresence>
        {isOpen && (
          <>
            {/* Overlay */}
            <motion.div
              className="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm md:hidden"
              onClick={() => setIsOpen(false)}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            ></motion.div>

            {/* Sidebar */}
            <motion.div
              className="fixed inset-y-0 left-0 z-50 w-64 bg-background h-screen overflow-y-auto border-r md:hidden"
              initial={{ x: "-100%" }}
              animate={{ x: 0 }}
              exit={{ x: "-100%" }}
              transition={{ type: "tween", duration: 0.3 }}
            >
              <div className="flex flex-col h-full pt-4">
                <NavItems />
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  )
}

export default NavBar

type MenuItemProps = {
  icon: React.ElementType
  title: string
  to: string
  notificationDot?: boolean
}

const MenuItem: React.FC<MenuItemProps> = ({
  to,
  title,
  icon: Icon,
  notificationDot = false,
}) => {
  const pathname = usePathname() || ""
  const selected = pathname.startsWith(to)
  return (
    <Link
      href={to}
      className={classNames(
        "p-2 flex items-center rounded-sm transition-colors duration-200",
        {
          "bg-secondary": selected,
          "hover:bg-secondary/50": !selected,
        },
      )}
    >
      <div className="relative mr-3">
        <Icon
          className={classNames("w-5 h-5", {
            "text-foreground": !selected,
            "text-primary": selected,
          })}
        />
        {notificationDot && (
          <span className="absolute top-0 right-0 w-2 h-2 bg-destructive rounded-full"></span>
        )}
      </div>
      <span
        className={classNames("text-sm", {
          "text-foreground": !selected,
          "text-primary font-medium": selected,
        })}
      >
        {title}
      </span>
    </Link>
  )
}
