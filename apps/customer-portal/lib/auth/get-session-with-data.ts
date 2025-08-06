import { auth } from "@/auth"
import { meQuery } from "@/lib/graphql/query/me"

export async function getSessionWithData() {
  try {
    const session = await auth()
    if (!session?.user) {
      return { session: null, meData: null }
    }
    const meData = await meQuery()
    return {
      session,
      meData: meData instanceof Error ? null : meData,
    }
  } catch (error) {
    console.error("Error fetching session with data:", error)
    return { session: null, meData: null }
  }
}
