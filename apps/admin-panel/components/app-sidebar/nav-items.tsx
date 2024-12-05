import {
  Home,
  Mouse,
  Users,
  LayoutGrid,
  ClipboardList,
  FileText,
  UserCircle,
  ArrowDownCircle,
  ArrowUpCircle,
  Globe,
  PieChart,
  DollarSign,
  LineChart,
  Users2,
  GanttChart,
  History,
  Scale,
} from "lucide-react"
import type { NavItem } from "./nav-section"

export const navDashboardItems: NavItem[] = [
  { title: "Dashboard", url: "/dashboard", icon: Home },
  { title: "Actions", url: "/actions", icon: Mouse },
]

export const navLoansItems: NavItem[] = [
  { title: "Term Templates", url: "/terms-templates", icon: FileText },
  { title: "Credit Facilities", url: "/credit-facilities", icon: LayoutGrid },
  { title: "Disbursals", url: "/disbursals", icon: ClipboardList },
]

export const navCustomersItems: NavItem[] = [
  { title: "Customers", url: "/customers", icon: Users },
  { title: "Policies", url: "/policies", icon: GanttChart },
  { title: "Regulatory Reporting", url: "/regulatory-reporting/customers", icon: Scale },
]

export const navTransactionItems: NavItem[] = [
  { title: "Deposits", url: "/deposits", icon: ArrowDownCircle },
  { title: "Withdrawals", url: "/withdrawals", icon: ArrowUpCircle },
]

export const navAdminItems: NavItem[] = [
  { title: "Users", url: "/users", icon: UserCircle },
  { title: "Commities", url: "/committees", icon: Users2 },
  { title: "Audit Logs", url: "/audit", icon: History },
]

export const navFinanceItems: NavItem[] = [
  { title: "Chart of Accounts", url: "/chart-of-accounts", icon: Globe },
  { title: "Balance Sheet", url: "/balance-sheet", icon: PieChart },
  { title: "Profit & Loss", url: "/profit-and-loss", icon: DollarSign },
  { title: "Trial Balance", url: "/trial-balance", icon: LineChart },
]
