"use client"

import { useState, useRef, useEffect } from "react"
import { motion, AnimatePresence } from "framer-motion"

import { Button, ID } from "@/components"
import Pill from "@/components/pill"

const animationProps = {
  initial: { opacity: 0, y: -10 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -10 },
  transition: { duration: 0.2 },
}

const Avatar = () => {
  const userName = "John Doe"
  const userEmail = "johndoe@example.net"
  const userRoles = ["Admin", "Bank Manager"]
  const userId = "3b6554e6-108b-494d-9791-58de1365b74a"
  const userRef = "10"

  const [showingDetails, setShowingDetails] = useState(false)
  const detailsRef = useRef<HTMLDivElement>(null)
  const avatarRef = useRef<HTMLDivElement>(null)

  const userNameInitials = userName
    .split(" ")
    .map((name) => name[0])
    .join("")
    .slice(0, 2)

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        detailsRef.current &&
        !detailsRef.current.contains(event.target as Node) &&
        avatarRef.current &&
        !avatarRef.current.contains(event.target as Node)
      ) {
        setShowingDetails(false)
      }
    }
    document.addEventListener("mousedown", handleClickOutside)
    return () => document.removeEventListener("mousedown", handleClickOutside)
  }, [])

  const Details = () => (
    <motion.div
      {...animationProps}
      ref={detailsRef}
      onClick={(e) => e.stopPropagation()}
      className="absolute top-12 right-0 bg-page shadow-md p-4 rounded-sm w-[200px] cursor-default flex flex-col space-y-1 items-start justify-center z-50"
    >
      <div className="flex flex-wrap gap-2">
        {userRoles.map((role) => (
          <Pill className="!text-[10px] py-0" key={role} color="brown" border>
            {role}
          </Pill>
        ))}
      </div>
      <div className="flex items-center justify-center space-x-2">
        <div className="text-title-md">{userName}</div>
        <div className="text-title-sm">#{userRef}</div>
      </div>
      <div className="text-body-sm">{userEmail}</div>
      <ID id={userId} />
      <div className="h-2"></div>
      <Button title="Logout" size="sm" />
    </motion.div>
  )

  return (
    <div
      ref={avatarRef}
      className="relative rounded-full bg-action center !h-10 !w-10 hover:bg-action-hover cursor-pointer hover:shadow"
      onClick={() => setShowingDetails((prev) => !prev)}
    >
      <span className="text-title-md !text-on-action select-none">
        {userNameInitials}
      </span>
      <AnimatePresence>{showingDetails && <Details />}</AnimatePresence>
    </div>
  )
}

export default Avatar
