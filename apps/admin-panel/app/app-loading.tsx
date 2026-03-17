"use client"

import React, { createContext, useContext, useState, useEffect } from "react"

import Logo from "@/public/lana-logo.svg"

const Loading = () => (
  <div
    className="flex items-center justify-center w-full h-full"
    data-testid="global-loader"
  >
    <Logo className="h-10 w-auto animate-pulse" />
  </div>
)

type LoadingContextType = {
  stopAppLoadingAnimation: () => void
}

const LoadingContext = createContext<LoadingContextType | undefined>(undefined)

export const AppLoading: React.FC<React.PropsWithChildren> = ({ children }) => {
  const [isLoading, setIsLoading] = useState(true)
  // Controls whether loader is rendered in the DOM at all
  const [showLoader, setShowLoader] = useState(true)

  const stopAppLoadingAnimation = () => setIsLoading(false)

  // Whenever isLoading becomes false, start a timer to remove the loader
  useEffect(() => {
    if (!isLoading) {
      const timeout = setTimeout(() => {
        setShowLoader(false)
      }, 500) // match your transition duration
      return () => clearTimeout(timeout)
    } else {
      // If you ever toggle isLoading back on, show the loader again
      setShowLoader(true)
    }
  }, [isLoading])

  return (
    <LoadingContext.Provider value={{ stopAppLoadingAnimation }}>
      {/* Loader container (conditional render) */}
      {showLoader && (
        <div
          className={`
            fixed inset-0 z-50 flex items-center justify-center
            bg-background transition-opacity duration-500 ease-in-out
            ${isLoading ? "opacity-100" : "opacity-0"}
          `}
        >
          <Loading />
        </div>
      )}

      {/* Main content container */}
      <div
        className={`
          transition-opacity duration-500 ease-in-out
          ${isLoading ? "opacity-0" : "opacity-100"}
        `}
      >
        {children}
      </div>
    </LoadingContext.Provider>
  )
}

export const useAppLoading = () => {
  const context = useContext(LoadingContext)
  if (!context) {
    throw new Error("useAppLoading must be used within an AppLoading provider")
  }
  return { stopAppLoadingAnimation: context.stopAppLoadingAnimation }
}

export default AppLoading
