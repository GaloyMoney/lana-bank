import { kratosPublic } from "@/lib/kratos/sdk"

import {
  emailParserFromUiNodeLogin,
  emailParserFromUiNodeRegister,
  getCsrfToken,
} from "@/lib/kratos/utils"

type SubmitAuthData = {
  flowId: string
  otp: string
  type: "login" | "register"
}
export const submitAuthFlow = async ({ flowId, otp, type }: SubmitAuthData) => {
  if (type === "register") {
    const flow = await kratosPublic.getRegistrationFlow({ id: flowId })

    const csrf_token = getCsrfToken(flow.data)
    if (!csrf_token) throw new Error("Kratos API didn't send CSRF token")

    const { data } = await kratosPublic.updateRegistrationFlow({
      flow: flowId,
      updateRegistrationFlowBody: {
        method: "code",
        code: otp,
        traits: {
          email: emailParserFromUiNodeRegister(flow.data.ui.nodes),
        },
        csrf_token,
      },
    })

    return data
  } else if (type === "login") {
    const flow = await kratosPublic.getLoginFlow({ id: flowId })

    const csrf_token = getCsrfToken(flow.data)
    if (!csrf_token) throw new Error("Kratos API didn't send CSRF token")

    const email = emailParserFromUiNodeLogin(flow.data.ui.nodes)
    if (!email) throw new Error("Email not found in the flow")

    const { data } = await kratosPublic.updateLoginFlow({
      flow: flowId,
      updateLoginFlowBody: {
        method: "code",
        code: otp,
        identifier: email,
        csrf_token,
      },
    })

    return data
  }
}
