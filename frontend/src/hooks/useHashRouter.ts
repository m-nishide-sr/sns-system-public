/**
 * ハッシュルーティング前提の画面遷移を扱うための軽量ルーター。
 */
import { useCallback, useEffect, useState } from "react";

/** 画面遷移で受け入れるハッシュルート一覧。 */
export const ROUTES = ["/", "/chat", "/login", "/mypage", "/terms"] as const;
/** `ROUTES` から導出されるアプリケーションの有効ルート。 */
export type AppRoute = (typeof ROUTES)[number];

/**
 * 任意のハッシュ文字列をアプリケーションで扱えるルートへ正規化する。
 */
export function normalizeHash(hash: string): AppRoute {
  const withoutHash = hash.replace(/^#/, "");
  const path = withoutHash === "" || withoutHash === "/" ? "/" : withoutHash.startsWith("/") ? withoutHash : `/${withoutHash}`;
  return (ROUTES as readonly string[]).includes(path) ? (path as AppRoute) : "/";
}

/** ブラウザの現在ハッシュからルートを求める。 */
function getCurrentRoute() {
  if (typeof window === "undefined") {
    return "/" as AppRoute;
  }

  return normalizeHash(window.location.hash);
}

/**
 * `hashchange` を購読し、現在ルートと遷移関数を返す。
 */
export function useHashRouter() {
  const [route, setRoute] = useState<AppRoute>(getCurrentRoute);

  useEffect(() => {
    const handleHashChange = () => {
      setRoute(getCurrentRoute());
    };

    if (!window.location.hash) {
      window.location.hash = "/";
    }

    handleHashChange();
    window.addEventListener("hashchange", handleHashChange);

    return () => window.removeEventListener("hashchange", handleHashChange);
  }, []);

  const navigate = useCallback((nextRoute: AppRoute) => {
    if (typeof window === "undefined") {
      return;
    }

    window.location.hash = nextRoute;
  }, []);

  return { route, navigate };
}
