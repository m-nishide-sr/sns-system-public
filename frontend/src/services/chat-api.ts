import type { ChatMessage } from "@/src/types/app";
import { fetchAccessToken } from "@/src/services/auth";

type ChatApiResponse =
  | ChatMessage[]
  | {
      items?: ChatMessage[];
    };

async function request(input: RequestInfo, init?: RequestInit) {
  const token = await fetchAccessToken();

  if (!token) {
    throw new Error("チャット機能を利用するにはログインが必要です。");
  }

  const response = await fetch(input, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      Authorization: 'Bearer ' + token,
      ...(init?.headers ?? {}),
    },
  });

  if (!response.ok) {
    throw new Error("APIの呼び出しに失敗しました。時間をおいて再試行してください。");
  }

  return response;
}

export async function fetchChats() {
  const response = await request("/api/chat");
  const data = (await response.json()) as ChatApiResponse;

  return Array.isArray(data) ? data : data.items ?? [];
}

export async function postChat(body: string) {
  const response = await request("/api/chat", {
    method: "POST",
    body: JSON.stringify({ body }),
  });

  const data = (await response.json().catch(() => null)) as
    | ChatMessage
    | null;

  return data;
}
