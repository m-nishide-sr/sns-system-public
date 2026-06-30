export type RouteKey = "home" | "login" | "chat" | "mypage" | "terms";

export type ChatMessage = {
  id: string;
  body: string;
  createdAt: string;
};

export type ToastState = {
  id: number;
  kind: "success" | "error";
  message: string;
};

export type AuthUser = {
  displayName: string;
  email?: string;
};
