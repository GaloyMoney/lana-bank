"use server"
import { redirect } from "next/navigation"

export const handleCustomerSearchFormSubmit = async (formData: FormData) => {
  if (formData.get("submit") === "clear") {
    redirect(`/customers`)
  }

  const customerId = formData.get("customerId")
  if (!customerId || typeof customerId !== "string") {
    redirect(`/customers`)
  }
  redirect(`/customers?customerId=${customerId}`)
}
