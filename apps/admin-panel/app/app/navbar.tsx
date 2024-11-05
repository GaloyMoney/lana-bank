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
} from "react-icons/hi"
import { HiMagnifyingGlassCircle } from "react-icons/hi2"

import { Logo } from "@/components"
import Avatar from "./avatar"

const NavBar = () => {
  const [isOpen, setIsOpen] = useState(false)

  const NavItems = () => (
    <nav className="m-4">
      <ul className="flex flex-col space-y-[1px]">
        <MenuItem icon={HiHome} title="Dashboard" to="/app/dashboard" />
        <MenuItem
          icon={HiCursorClick}
          title="Actions"
          to="/app/actions"
          notificationDot
        />
        <MenuItem icon={HiMagnifyingGlassCircle} title="Search" to="/app/search" />
        <div className="h-4"></div>
        <MenuItem icon={HiUser} title="Customers" to="/app/customers" />
        <MenuItem
          icon={HiViewGrid}
          title="Credit Facilities"
          to="/app/credit-facilities"
        />
        <MenuItem icon={HiViewBoards} title="Disbursals" to="/app/disbursals" />
        <MenuItem icon={HiTemplate} title="Terms Templates" to="/app/terms-templates" />
        <MenuItem icon={HiUserGroup} title="Users" to="/app/users" />
        <MenuItem icon={HiGlobe} title="Committees" to="/app/committees" />
        <MenuItem icon={HiArrowCircleDown} title="Deposits" to="/app/deposits" />
        <MenuItem icon={HiArrowCircleUp} title="Withdrawals" to="/app/withdrawals" />
        <MenuItem
          icon={HiDocumentReport}
          title="Regulatory Reporting"
          to="/app/regulatory-reporting"
        />
        <div className="h-4"></div>
        <MenuItem
          icon={HiGlobeAlt}
          title="Chart of Accounts"
          to="/app/chart-of-accounts"
        />
        <MenuItem icon={HiCurrencyDollar} title="Balance Sheet" to="/app/balance-sheet" />
        <MenuItem icon={HiCash} title="Profit & Loss" to="/app/profit-and-loss" />
        <MenuItem icon={HiLightningBolt} title="Trial Balance" to="/app/trial-balance" />
      </ul>
    </nav>
  )

  return (
    <div className="md:w-64">
      {/* Mobile Top Bar */}
      <div className="fixed top-0 left-0 right-0 z-50 shadow bg-soft md:hidden">
        <div className="flex items-center justify-between px-4 py-2">
          <button onClick={() => setIsOpen(true)} aria-label="Toggle navigation menu">
            <HiOutlineMenu className="w-8 h-8 text-neutral-800" />
          </button>
          <Logo width={21} />
          <Avatar />
        </div>
      </div>

      {/* Desktop Sidebar */}
      <div className="hidden md:flex md:flex-col min-h-screen justify-between">
        <div>
          <div className="flex justify-between items-center w-full p-5">
            <Logo />
            <Avatar />
          </div>
          <NavItems />
        </div>
        <div className="p-5 text-body-sm">USD/BTC Market Rate: $60,000.12</div>
      </div>

      {/* Mobile Sidebar */}
      <AnimatePresence>
        {isOpen && (
          <>
            {/* Overlay */}
            <motion.div
              className="fixed inset-0 z-40 bg-black bg-opacity-50 md:hidden"
              onClick={() => setIsOpen(false)}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            ></motion.div>

            {/* Sidebar */}
            <motion.div
              className="fixed inset-y-0 left-0 z-50 w-64 bg-soft h-screen overflow-y-auto shadow-lg md:hidden"
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
  const selected = pathname === to
  return (
    <Link
      href={to}
      className={classNames(
        "p-2 flex items-center rounded-sm transition-colors duration-200",
        {
          "bg-action-secondary": selected,
          "hover:bg-action-secondary-hover": !selected,
        },
      )}
    >
      <div className="relative mr-3">
        <Icon
          className={classNames("w-5 h-5", {
            "text-heading": !selected,
            "text-on-action": selected,
          })}
        />
        {notificationDot && (
          <span className="absolute top-0 right-0 w-2 h-2 bg-red-500 rounded-full"></span>
        )}
      </div>
      <span
        className={classNames("text-body-md", {
          "text-heading": !selected,
          "!text-on-action": selected,
        })}
      >
        {title}
      </span>
    </Link>
  )
}
