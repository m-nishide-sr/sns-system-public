import type { RouteKey } from "@/src/types/app";

type RouteDefinition = {
  title: string;
  showHeader: boolean;
};

export const routeDefinitions: Record<RouteKey, RouteDefinition> = {
  home: {
    title: "SNSシステム",
    showHeader: false,
  },
  login: {
    title: "ログイン / 新規登録",
    showHeader: true,
  },
  chat: {
    title: "チャット",
    showHeader: true,
  },
  mypage: {
    title: "マイページ",
    showHeader: true,
  },
  terms: {
    title: "利用規約",
    showHeader: true,
  },
};

const routeMap: Record<string, RouteKey> = {
  "": "home",
  "#": "home",
  "#/": "home",
  "#/login": "login",
  "#/chat": "chat",
  "#/mypage": "mypage",
  "#/terms": "terms",
};

export function resolveRoute(hash: string): RouteKey {
  return routeMap[hash] ?? "home";
}

export function buildHash(route: RouteKey): string {
  if (route === "home") {
    return "#/";
  }

  return `#/${route}`;
}
