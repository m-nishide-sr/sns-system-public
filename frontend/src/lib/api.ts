import { fetchAuthSession } from "aws-amplify/auth";
import type { components, operations } from "@/types/api";

export type TimelineMessage = components["schemas"]["TimelineMessageResponse"];
export type CreateMessageRequest = components["schemas"]["CreateMessageRequest"];
export type CreateMessageResponse = components["schemas"]["CreateMessageResponse"];
export type TimelineQuery = operations["timeline_endpoint_doc"]["parameters"]["query"];

const API_BASE = "/api/v1";

export class ApiError extends Error {
  status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

async function getIdToken() {
  const session = await fetchAuthSession();
  const token = session.tokens?.idToken?.toString();

  if (!token) {
    throw new ApiError("ログインが必要です。", 401);
  }

  return token;
}

async function request<T>(path: string, init?: RequestInit) {
  const token = await getIdToken();
  const headers = new Headers(init?.headers);
  headers.set("Content-Type", "application/json");
  const authorizationValue = ["Bearer", token].join(" ");
  headers.set("Authorization", authorizationValue);

  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers,
  });

  if (!response.ok) {
    let message = `API呼び出しに失敗しました。（${response.status}）`;

    try {
      const body = (await response.json()) as { message?: string };
      if (body.message) {
        message = body.message;
      }
    } catch {
      // JSONでない場合は既定メッセージを使う。
    }

    throw new ApiError(message, response.status);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

export async function fetchTimeline(before?: TimelineQuery extends { before?: infer T } ? T : string) {
  const search = before ? `?before=${encodeURIComponent(before)}` : "";
  return request<TimelineMessage[]>(`/timeline${search}`, { method: "GET" });
}

export async function postMessage(payload: CreateMessageRequest) {
  return request<CreateMessageResponse>("/message", {
    method: "POST",
    body: JSON.stringify(payload),
  });
}
