import { useCallback, useEffect, useState } from "react";

export const ROUTES = ["/", "/chat", "/login", "/mypage", "/terms"] as const;
export type AppRoute = (typeof ROUTES)[number];

export function normalizeHash(hash: string): AppRoute {
  const withoutHash = hash.replace(/^#/, "");
  const path = withoutHash === "" || withoutHash === "/" ? "/" : withoutHash.startsWith("/") ? withoutHash : `/${withoutHash}`;
  return (ROUTES as readonly string[]).includes(path) ? (path as AppRoute) : "/";
}

function getCurrentRoute() {
  if (typeof window === "undefined") {
    return "/" as AppRoute;
  }

  return normalizeHash(window.location.hash);
}

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
