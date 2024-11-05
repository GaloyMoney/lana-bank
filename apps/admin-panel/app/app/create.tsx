/* eslint-disable no-empty-function */
"use client"

import { useState, useRef, useEffect } from "react"
import { HiPlus } from "react-icons/hi"

import { motion, AnimatePresence } from "framer-motion"

import CreateCustomer from "./customers/create"

import { Button } from "@/components"

const CreateButton = () => {
  const [openCreateCustomer, setOpenCreateCustomer] = useState(false)

  const [isOpen, setIsOpen] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    document.addEventListener("mousedown", handleClickOutside)
    return () => document.removeEventListener("mousedown", handleClickOutside)
  }, [])

  const menuItems = [
    { label: "Disbursal", onClick: () => {} },
    { label: "Deposit", onClick: () => {} },
    { label: "Withdrawal", onClick: () => {} },
    { label: "Customer", onClick: () => setOpenCreateCustomer(true) },
    { label: "Credit Facility", onClick: () => {} },
  ]

  return (
    <>
      <div className="relative inline-block" ref={menuRef}>
        <Button
          size="md"
          title="Create"
          icon={<HiPlus className="text-lg" />}
          className="py-3 px-6 w-36"
          onClick={() => setIsOpen(!isOpen)}
        />

        <AnimatePresence>
          {isOpen && (
            <motion.div
              className="absolute right-0 mt-2 w-36 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 z-50"
              initial={{ opacity: 0, scale: 0.95, y: -10 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.95, y: -10 }}
              transition={{ duration: 0.2 }}
            >
              <div className="py-1">
                {menuItems.map((item, index) => (
                  <button
                    key={index}
                    onClick={() => {
                      item.onClick()
                      setIsOpen(false)
                    }}
                    className="block w-full text-left px-4 py-2 text-sm text-title-sm hover:bg-action-secondary-hover"
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
      <CreateCustomer open={openCreateCustomer} setOpen={setOpenCreateCustomer} />
    </>
  )
}

export default CreateButton
