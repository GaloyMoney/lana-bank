"use client"

import { useState, useRef, useEffect } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { gql } from "@apollo/client"
import { signOut } from "next-auth/react"

import { Button } from "@/components/primitive/button"
import { useAvatarQuery } from "@/lib/graphql/generated"
import { ID, Pill } from "@/components/new"

const animationProps = {
  initial: { opacity: 0, y: -10 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -10 },
  transition: { duration: 0.2 },
}

gql`
  query Avatar {
    me {
      user {
        userId
        email
        roles
      }
    }
  }
`

const Avatar = () => {
  const { data, loading } = useAvatarQuery()

  const [showingDetails, setShowingDetails] = useState(false)
  const detailsRef = useRef<HTMLDivElement>(null)
  const avatarRef = useRef<HTMLDivElement>(null)

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

  if (!data || loading) return <></>

  const userName = data.me.user.email.split("@")[0]
  const userEmail = data.me.user.email
  const userRoles = data.me.user.roles
  const userId = data.me.user.userId
  const userRef = ""

  const userNameInitials = userName
    .split(" ")
    .map((name) => name[0])
    .join("")
    .slice(0, 2)

  const Details = () => (
    <motion.div
      {...animationProps}
      ref={detailsRef}
      onClick={(e) => e.stopPropagation()}
      className="absolute top-12 right-0 bg-popover text-popover-foreground shadow-lg p-4 rounded-md w-[200px] cursor-default flex flex-col space-y-1 items-start justify-center z-10 border"
    >
      <div className="flex flex-wrap gap-2">
        {userRoles.map((role) => (
          <Pill className="!text-[10px] py-0" key={role} color="brown" border>
            {role}
          </Pill>
        ))}
      </div>
      <div className="flex items-center justify-center space-x-2">
        <div className="text-base font-medium capitalize">{userName}</div>
        {userRef && <div className="text-sm font-medium">#{userRef}</div>}
      </div>
      <div className="text-sm text-muted-foreground">{userEmail}</div>
      <ID type="Your" id={userId} />
      <div className="h-2"></div>
      <Button
        variant="destructive"
        size="sm"
        onClick={() => signOut()}
        className="w-full"
      >
        Logout
      </Button>
    </motion.div>
  )

  return (
    <div
      ref={avatarRef}
      className="relative flex h-10 w-10 items-center justify-center rounded-full bg-primary hover:bg-primary/90 cursor-pointer transition-colors"
      onClick={() => setShowingDetails((prev) => !prev)}
    >
      <span className="select-none text-base font-semibold text-primary-foreground uppercase">
        {userNameInitials}
      </span>
      <AnimatePresence>{showingDetails && <Details />}</AnimatePresence>
    </div>
  )
}

export default Avatar
