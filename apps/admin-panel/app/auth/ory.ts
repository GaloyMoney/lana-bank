"use client"

/* eslint-disable camelcase */ // Many Ory Kratos request body params are snake_case

import { Configuration, FrontendApi, UiNodeInputAttributes } from "@ory/client"
import axios, { AxiosError } from "axios"

export const getOryClient = () =>
  new FrontendApi(
    new Configuration({
      baseOptions: {
        withCredentials: true,
        timeout: 10000,
      },
    }),
    "",
    axios,
  )

export const getSession = () => {
  const kratos = getOryClient()
  return kratos.toSession()
}

export const loginUser = async (email: string) => {
  const oryClient = getOryClient()
  let flowId: string = ""

  try {
    const { data: flow } = await oryClient.createBrowserLoginFlow()
    flowId = flow.id

    const { data: loginData } = await oryClient.updateLoginFlow({
      flow: flow.id,
      updateLoginFlowBody: {
        method: "code",
        identifier: email,
        csrf_token:
          (
            flow.ui.nodes.find(
              (node) =>
                node.attributes.node_type === "input" &&
                node.attributes.name === "csrf_token",
            )?.attributes as UiNodeInputAttributes
          ).value || "",
      },
    })
    return loginData
  } catch (error) {
    if (
      error instanceof AxiosError &&
      error.code === AxiosError.ERR_BAD_REQUEST &&
      error.response?.data.ui.messages[0].id === 1010014
    ) {
      return flowId
    }
    throw error
  }
}

export const loginUserWithOtp = async (flowId: string, otp: string) => {
  const oryClient = getOryClient()

  const { data: loginFlow } = await oryClient.getLoginFlow({
    id: flowId,
  })

  const csrf_token =
    (
      loginFlow.ui.nodes.find(
        (node) =>
          node.attributes.node_type === "input" && node.attributes.name === "csrf_token",
      )?.attributes as UiNodeInputAttributes
    ).value || ""

  const identifier =
    (
      loginFlow.ui.nodes.find(
        (node) =>
          node.attributes.node_type === "input" && node.attributes.name === "identifier",
      )?.attributes as UiNodeInputAttributes
    ).value || ""

  const { data: loginData } = await oryClient.updateLoginFlow({
    flow: flowId,
    updateLoginFlowBody: {
      method: "code",
      identifier,
      csrf_token,
      code: otp,
    },
  })

  return loginData
}

export const logoutUser = async () => {
  const oryClient = getOryClient()
  const { data } = await oryClient.createBrowserLogoutFlow()
  await oryClient.updateLogoutFlow({
    token: data.logout_token,
  })
}
