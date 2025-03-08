"use server"

import { sumsubPermalinkCreate } from "@/lib/graphql/mutation/sumsub"

export async function createSumsubPermalink() {
        console.log("createSumsubPermalink")
        const data = await sumsubPermalinkCreate()
        console.log("data", data)
        if (data instanceof Error) {
                throw data
        }
        return data.sumsubPermalinkCreate
}
