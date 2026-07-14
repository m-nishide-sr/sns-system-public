"use client";

/**
 * ハッシュルーティング SPA の画面切り替えと認証状態を束ねるトップレベルページ。
 */
import { useCallback, useEffect, useMemo, useState } from "react";
import { fetchAuthSession } from "aws-amplify/auth";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { Menu } from "@/components/Menu";
import { ToastViewport, useToast } from "@/components/Toast";
import { ChatPage } from "@/components/pages/ChatPage";
import { LoginPage } from "@/components/pages/LoginPage";
import { MyPage } from "@/components/pages/MyPage";
import { TermsPage } from "@/components/pages/TermsPage";
import { TopPage } from "@/components/pages/TopPage";
import { ensureAmplifyConfigured } from "@/lib/amplify";
import { useHashRouter, type AppRoute } from "@/hooks/useHashRouter";

ensureAmplifyConfigured();

const PAGE_TITLES: Record<AppRoute, string> = {
  "/": "トップ",
  "/chat": "チャット",
  "/login": "ログイン / 新規登録",
  "/mypage": "マイページ",
  "/terms": "利用規約",
};

/** 認証済みセッションが存在するかどうかだけを簡易判定する。 */
async function hasAuthenticatedSession() {
  try {
    const session = await fetchAuthSession();
    return Boolean(session.tokens?.idToken);
  } catch {
    return false;
  }
}

/** ルートごとの画面と共通 UI を組み立てる SPA エントリーポイント。 */
export default function Home() {
  const { route, navigate } = useHashRouter();
  const { toasts, showToast, removeToast } = useToast();
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isAuthChecking, setIsAuthChecking] = useState(true);

  const refreshAuthState = useCallback(async () => {
    setIsAuthChecking(true);
    const nextState = await hasAuthenticatedSession();
    setIsAuthenticated(nextState);
    setIsAuthChecking(false);
    return nextState;
  }, []);

  useEffect(() => {
    let isMounted = true;

    const checkAuthState = async () => {
      const nextState = await hasAuthenticatedSession();
      if (!isMounted) {
        return;
      }
      setIsAuthenticated(nextState);
      setIsAuthChecking(false);
    };

    void checkAuthState();

    return () => {
      isMounted = false;
    };
  }, []);

  const handleNavigate = useCallback(
    (nextRoute: AppRoute) => {
      setIsMenuOpen(false);
      navigate(nextRoute);
    },
    [navigate],
  );

  const pageTitle = PAGE_TITLES[route];
  const showHeader = route !== "/";

  const page = useMemo(() => {
    switch (route) {
      case "/chat":
        return (
          <ChatPage
            isAuthenticated={isAuthenticated}
            isAuthChecking={isAuthChecking}
            onNavigate={handleNavigate}
            onToast={showToast}
          />
        );
      case "/login":
        return (
          <LoginPage
            isAuthenticated={isAuthenticated}
            onAuthSuccess={async () => {
              await refreshAuthState();
              handleNavigate("/chat");
            }}
            onNavigate={handleNavigate}
            onToast={showToast}
          />
        );
      case "/mypage":
        return (
          <MyPage
            isAuthenticated={isAuthenticated}
            isAuthChecking={isAuthChecking}
            onAuthChange={refreshAuthState}
            onNavigate={handleNavigate}
            onToast={showToast}
          />
        );
      case "/terms":
        return <TermsPage onNavigate={handleNavigate} />;
      case "/":
      default:
        return (
          <TopPage
            isAuthenticated={isAuthenticated}
            onNavigate={handleNavigate}
          />
        );
    }
  }, [handleNavigate, isAuthChecking, isAuthenticated, refreshAuthState, route, showToast]);

  return (
    <div className="min-h-screen bg-[radial-gradient(circle_at_top,_#eff6ff_0%,_#f8fafc_35%,_#f8fafc_100%)]">
      {showHeader ? (
        <Header
          isAuthenticated={isAuthenticated}
          title={pageTitle}
          onLoginClick={() => handleNavigate("/login")}
          onMenuToggle={() => setIsMenuOpen((current) => !current)}
          onMyPageClick={() => handleNavigate("/mypage")}
        />
      ) : null}
      <Menu
        currentRoute={route}
        isAuthenticated={isAuthenticated}
        isOpen={isMenuOpen}
        onClose={() => setIsMenuOpen(false)}
        onNavigate={handleNavigate}
      />
      <div className="flex min-h-screen flex-col">
        <main className={showHeader ? "flex-1 px-4 pb-12 pt-24 sm:px-6 lg:px-8" : "flex-1 px-4 py-10 sm:px-6 lg:px-8"}>
          {page}
        </main>
        <Footer />
      </div>
      <ToastViewport toasts={toasts} onClose={removeToast} />
    </div>
  );
}
