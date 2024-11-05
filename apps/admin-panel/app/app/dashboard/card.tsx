"use client"

import Link from "next/link"
import { HiArrowRight } from "react-icons/hi"
import { motion, AnimatePresence } from "framer-motion"

import { useState } from "react"

import { Button } from "@/components"

type DashboardCardProps = {
  h1?: string
  h2?: string
  h2PopupDescription?: string
  title: string
  description: string
  to?: string
  buttonToRight?: boolean
  content?: React.ReactElement
  buttonText?: string
}

const DashboardCard: React.FC<DashboardCardProps> = ({
  h1,
  h2,
  h2PopupDescription,
  title,
  description,
  to = "",
  content,
  buttonToRight = false,
  buttonText = "",
}) => {
  const [isHovered, setIsHovered] = useState(false)

  return (
    <div className="bg-page cursor-default p-[10px] rounded-md w-full flex flex-col items-start gap-1 !text-heading relative">
      <div className="flex items-end gap-2">
        {h1 && <div className="text-heading-h5">{h1}</div>}
        {h2 && (
          <div
            className="relative text-heading-h6 !text-gray-500 mb-[1px]"
            onMouseEnter={() => setIsHovered(true)}
            onMouseLeave={() => setIsHovered(false)}
          >
            {h2}
            <AnimatePresence>
              {isHovered && h2PopupDescription && (
                <motion.div
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -10 }}
                  transition={{ duration: 0.2 }}
                  className="absolute left-full -top-[1px] transform -translate-y-1/2 ml-2 bg-gray-800 text-white text-xs px-3 py-1 rounded-md z-10 whitespace-nowrap"
                >
                  {h2PopupDescription}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        )}
      </div>
      <div className="text-title-md">{title}</div>
      <div className="!text-body text-body-sm">{description}</div>
      {content}
      {to && (
        <Link href={to} className={buttonToRight ? "self-end" : ""}>
          <Button
            className="mt-2"
            title={buttonText || "View Details"}
            size="sm"
            variant="outlined"
            rightIcon={buttonText ? undefined : <HiArrowRight className="text-lg" />}
          />
        </Link>
      )}
    </div>
  )
}

export default DashboardCard
