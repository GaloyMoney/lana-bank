"use client"
import Link from "next/link"
import { usePathname } from "next/navigation"

import { User, Receipt, CashOutline } from "@/components/icons"

const navLinks = [
  { href: "/loan", label: "Loan", icon: Receipt },
  { href: "/user", label: "Users", icon: User },
  { href: "/trial-balance", label: "Trial Balance", icon: CashOutline },
]

const NavigationLinks = () => {
  const pathname = usePathname()
  return (
    <nav className="flex flex-col gap-6 text-textColor-secondary">
      {navLinks.map((link, index) => (
        <Link
          key={index}
          href={link.href}
          prefetch={false}
          className={`hover:text-textColor-primary ${pathname === link.href && "text-primary"}`}
        >
          <div className="flex items-center gap-4 rounded-md">
            <link.icon className="w-5 h-5" />
            {link.label}
          </div>
        </Link>
      ))}
    </nav>
  )
}

export { NavigationLinks }
