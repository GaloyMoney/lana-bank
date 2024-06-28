import { AxiosError } from "axios"

import { kratosPublic } from "@/lib/kratos/sdk"
import { getCsrfToken, kratosUiMessageIds } from "@/lib/kratos/utils"

import { OtpParams } from "@/components/auth/otp-form"

type IdentityTraits = {
  email: string
}
class InvalidFlowError extends Error {}
class UserNotExistError extends Error {}

const createLoginFlow = async ({ email }: IdentityTraits): Promise<OtpParams> => {
  const flow = await kratosPublic.createBrowserLoginFlow()
  const flowId = flow.data.id

  const csrf_token = getCsrfToken(flow.data)
  if (!csrf_token) throw new Error("Kratos API didn't send CSRF token")

  try {
    await kratosPublic.updateLoginFlow({
      flow: flowId,
      updateLoginFlowBody: {
        method: "code",
        identifier: email,
        csrf_token,
      },
    })
  } catch (error) {
    if (
      error instanceof AxiosError &&
      error.response?.data.ui.messages[0].id === kratosUiMessageIds.USER_NOT_EXIST
    ) {
      throw new UserNotExistError()
    } else if (
      error instanceof AxiosError &&
      error.response?.data.ui.messages[0].id === kratosUiMessageIds.OTP_EMAIL_SENT_SIGN_IN
    ) {
      // This is the only flow that is expected
      return { flowId: error.response.data.id, type: "login" }
    }
  }
  throw new InvalidFlow()
}

const createRegisterFlow = async ({ email }: IdentityTraits): Promise<OtpParams> => {
  const flow = await kratosPublic.createBrowserRegistrationFlow()
  const flowId = flow.data.id

  const csrf_token = getCsrfToken(flow.data)
  if (!csrf_token) throw new Error("Kratos API didn't send CSRF token")

  try {
    await kratosPublic.updateRegistrationFlow({
      flow: flowId,
      updateRegistrationFlowBody: {
        method: "code",
        traits: {
          email,
        },
        csrf_token,
      },
    })
  } catch (error) {
    if (
      error instanceof AxiosError &&
      error.response?.data.ui.messages[0].id ===
        kratosUiMessageIds.OTP_EMAIL_SENT_REGISTER
    ) {
      // This is the only flow that is expected
      return { flowId: error.response.data.id, type: "register" }
    }
  }
  throw new InvalidFlow()
}

export const createAuthFlow = async ({ email }: IdentityTraits): Promise<OtpParams> => {
  try {
    return await createLoginFlow({ email })
  } catch (error) {
    if (error instanceof UserNotExistError) return createRegisterFlow({ email })
  }
  throw new InvalidFlow()
}
