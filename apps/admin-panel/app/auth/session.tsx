"use client"

import {
  useEffect,
  useState,
  useCallback,
  createContext,
  useContext,
  useMemo,
  useRef,
} from "react"
import { usePathname, useRouter } from "next/navigation"
import dynamic from "next/dynamic"

import { ApolloProvider } from "@apollo/client"

import { AppLayout } from "../app-layout"

import { BreadcrumbProvider } from "../breadcrumb-provider"

import { useAppLoading } from "../app-loading"

import { getSession, logoutUser } from "./ory"

import { Toast } from "@/components/toast"

import { makeClient } from "@/lib/apollo-client/client"
import { env } from "@/env"

const AuthenticatedStore = createContext({
  // eslint-disable-next-line no-empty-function
  logoutInAuthState: () => {},
  // eslint-disable-next-line no-empty-function
  resetInactivityTimer: () => {},
})

type Props = {
  children: React.ReactNode
}

const AuthenticatedGuard: React.FC<Props> = ({ children }) => {
  const router = useRouter()
  const pathName = usePathname()
  const { stopAppLoadingAnimation } = useAppLoading()

  const [isAuthSetInLocalStorage, setIsAuthSetInLocalStorage] = useState(false)
  const [isAuthenticated, setIsAuthenticated] = useState<boolean | null>(null)

  const SESSION_TIMEOUT = 5 * 60 * 1000 // 5 minutes
  const inactivityTimerRef = useRef<NodeJS.Timeout | null>(null)

  const logoutInAuthState = useMemo(
    () => () => {
      setIsAuthSetInLocalStorage(false)
      setIsAuthenticated(false)
      if (typeof window !== "undefined") {
        localStorage.removeItem("isAuthenticated")
      }

      if (inactivityTimerRef.current) {
        clearTimeout(inactivityTimerRef.current)
        inactivityTimerRef.current = null
      }
    },
    [setIsAuthSetInLocalStorage, setIsAuthenticated],
  )

  const resetInactivityTimer = useCallback(() => {
    if (inactivityTimerRef.current) {
      clearTimeout(inactivityTimerRef.current)
    }

    if (isAuthenticated) {
      inactivityTimerRef.current = setTimeout(async () => {
        console.log("Session expired due to inactivity")
        await logoutUser()
        logoutInAuthState()
        router.push("/auth/login?reason=session_timeout")
      }, SESSION_TIMEOUT)
    }
  }, [isAuthenticated, logoutInAuthState, router, SESSION_TIMEOUT])

  useEffect(() => {
    if (typeof window !== "undefined") {
      const authFromLocalStorage = localStorage.getItem("isAuthenticated")
      setIsAuthSetInLocalStorage(!!authFromLocalStorage)
    }
  }, [])

  useEffect(() => {
    ;(async () => {
      try {
        await getSession()
        setIsAuthenticated(true)

        if (typeof window !== "undefined") {
          localStorage.setItem("isAuthenticated", "true")
        }
        if (pathName === "/") router.push("/dashboard")
      } catch (error) {
        setIsAuthenticated(false)
        if (typeof window !== "undefined") {
          localStorage.removeItem("isAuthenticated")
        }
        if (!pathName.startsWith("/auth")) router.push("/auth/login")
      } finally {
        stopAppLoadingAnimation()
      }
    })()
  }, [pathName, router, stopAppLoadingAnimation])

  // Set up event listeners to track user activity
  useEffect(() => {
    if (!isAuthenticated) return

    const activityEvents = ["mousedown", "mousemove", "keypress", "scroll", "touchstart"]

    const handleUserActivity = () => {
      resetInactivityTimer()
    }

    activityEvents.forEach((event) => {
      window.addEventListener(event, handleUserActivity)
    })

    resetInactivityTimer()

    return () => {
      if (inactivityTimerRef.current) {
        clearTimeout(inactivityTimerRef.current)
      }
      activityEvents.forEach((event) => {
        window.removeEventListener(event, handleUserActivity)
      })
    }
  }, [isAuthenticated, resetInactivityTimer])

  const appVersion = env.NEXT_PUBLIC_APP_VERSION
  const client = useMemo(() => {
    return makeClient({
      coreAdminGqlUrl: appVersion.endsWith("dev") ? "/admin/graphql" : "/graphql",
    })
  }, [appVersion])

  const Stack =
    isAuthenticated || (isAuthenticated === null && isAuthSetInLocalStorage) ? (
      // If we know the user is authenticated or is marked authenticated in localStorage
      <AppLayout>{children}</AppLayout>
    ) : (
      // Otherwise, just render the children (loading states, or unauthenticated routes)
      <main className="h-screen w-full flex flex-col">{children}</main>
    )

  return (
    <BreadcrumbProvider>
      <ApolloProvider client={client}>
        <AuthenticatedStore.Provider value={{ logoutInAuthState, resetInactivityTimer }}>
          <Toast />
          {Stack}
        </AuthenticatedStore.Provider>
      </ApolloProvider>
    </BreadcrumbProvider>
  )
}

export const Authenticated = dynamic(() => Promise.resolve(AuthenticatedGuard), {
  ssr: false,
})

export const useLogout = () => {
  const router = useRouter()
  const { logoutInAuthState } = useContext(AuthenticatedStore)

  const logout = useCallback(async () => {
    await logoutUser()
    logoutInAuthState()
    router.push("/")
  }, [router, logoutInAuthState])

  return { logout }
}
