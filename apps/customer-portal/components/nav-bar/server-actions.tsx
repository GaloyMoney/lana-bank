"use server"

import { sumsubPermalinkCreate } from "@/lib/graphql/mutation/sumsub"

export async function createSumsubPermalink() {
  const data = await sumsubPermalinkCreate()
  if (data instanceof Error) {
    throw data
  }
  return data.sumsubPermalinkCreate
}
