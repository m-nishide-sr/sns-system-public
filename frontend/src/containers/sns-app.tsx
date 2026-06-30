"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ChatSection,
  HomeSection,
  LoginSection,
  MyPageSection,
  TermsSection,
} from "@/src/components/page-sections";
import { SiteShell } from "@/src/components/site-shell";
import { buildHash, resolveRoute, routeDefinitions } from "@/src/lib/routes";
import {
  changePassword,
  confirmSignUpCode,
  deleteCurrentUser,
  fetchSignedInUser,
  normalizeAuthError,
  signInWithEmail,
  signOutCurrentUser,
  signUpWithEmail,
} from "@/src/services/auth";
import { configureAmplify, getFrontendConfig } from "@/src/services/amplify";
import { fetchChats, postChat } from "@/src/services/chat-api";
import type { ChatMessage, RouteKey, ToastState } from "@/src/types/app";

const toastTimeoutMs = 4000;

export function SnsApp() {
  const frontendConfig = useMemo(() => getFrontendConfig(), []);
  const [route, setRoute] = useState<RouteKey>("home");
  const [menuOpen, setMenuOpen] = useState(false);
  const [toasts, setToasts] = useState<ToastState[]>([]);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [userLabel, setUserLabel] = useState("未ログイン");
  const [signedIn, setSignedIn] = useState(false);
  const [signInEmail, setSignInEmail] = useState("");
  const [signInPassword, setSignInPassword] = useState("");
  const [signUpEmail, setSignUpEmail] = useState("");
  const [signUpPassword, setSignUpPassword] = useState("");
  const [confirmationEmail, setConfirmationEmail] = useState("");
  const [confirmationCode, setConfirmationCode] = useState("");
  const [currentPassword, setCurrentPassword] = useState("");
  const [nextPassword, setNextPassword] = useState("");
  const nextToastId = useRef(1);

  const definition = routeDefinitions[route];
  const allowDomain = frontendConfig.allowDomain ?? "許可されたドメイン";

  const actionLabel = signedIn ? "マイページ" : "ログイン";
  const actionHref = signedIn ? buildHash("mypage") : buildHash("login");

  const menuItems = [
    { href: buildHash("home"), label: "トップページ" },
    { href: buildHash("login"), label: "ログイン / 新規登録" },
    { href: buildHash("chat"), label: "チャット" },
    { href: buildHash("mypage"), label: "マイページ" },
    { href: buildHash("terms"), label: "利用規約" },
  ];

  const refreshUser = useCallback(async () => {
    const user = await fetchSignedInUser();

    if (user) {
      setUserLabel(user.displayName);
      setSignedIn(true);
      if (user.email) {
        setConfirmationEmail(user.email);
      }
      return;
    }

    setUserLabel("未ログイン");
    setSignedIn(false);
  }, []);

  const pushToast = useCallback((kind: ToastState["kind"], message: string) => {
    const id = nextToastId.current;
    nextToastId.current += 1;

    setToasts((current) => [...current, { id, kind, message }]);

    window.setTimeout(() => {
      setToasts((current) => current.filter((toast) => toast.id !== id));
    }, toastTimeoutMs);
  }, []);

  const refreshChats = useCallback(async () => {
    try {
      const chats = await fetchChats();
      setMessages(chats);
      pushToast("success", "チャットを更新しました。");
    } catch (error) {
      pushToast("error", normalizeAuthError(error));
    }
  }, [pushToast]);

  useEffect(() => {
    configureAmplify();

    const syncRoute = () => {
      setRoute(resolveRoute(window.location.hash));
    };

    syncRoute();
    window.addEventListener("hashchange", syncRoute);

    queueMicrotask(() => {
      void refreshUser();
    });

    return () => {
      window.removeEventListener("hashchange", syncRoute);
    };
  }, [refreshUser]);

  useEffect(() => {
    if (route === "chat" && signedIn) {
      const timer = window.setTimeout(() => {
        void refreshChats();
      }, 0);

      return () => {
        window.clearTimeout(timer);
      };
    }
  }, [refreshChats, route, signedIn]);

  async function handlePostChat() {
    if (!draft.trim()) {
      pushToast("error", "投稿内容を入力してください。");
      return;
    }

    try {
      await postChat(draft);
      setDraft("");
      pushToast("success", "チャットを投稿しました。");
      await refreshChats();
    } catch (error) {
      pushToast("error", normalizeAuthError(error));
    }
  }

  async function handleSignIn() {
    try {
      const result = await signInWithEmail(signInEmail, signInPassword);

      if (result.nextStep.signInStep === "DONE") {
        await refreshUser();
        window.location.hash = buildHash("chat");
        pushToast("success", "ログインしました。");
        return;
      }

      setConfirmationEmail(signInEmail);
      pushToast(
        "error",
        "確認コードの入力が必要です。メールをご確認ください。",
      );
    } catch (error) {
      const message = normalizeAuthError(error);
      if (message.includes("UserNotConfirmed")) {
        setConfirmationEmail(signInEmail);
      }
      pushToast("error", message);
    }
  }

  async function handleSignUp() {
    try {
      const result = await signUpWithEmail(signUpEmail, signUpPassword);
      setConfirmationEmail(signUpEmail);
      if (result.nextStep.signUpStep === "DONE") {
        pushToast("success", "登録が完了しました。ログインしてください。");
        return;
      }
      pushToast("success", "確認コードを送信しました。受信メールをご確認ください。");
    } catch (error) {
      pushToast("error", normalizeAuthError(error));
    }
  }

  async function handleConfirmSignUp() {
    try {
      await confirmSignUpCode(confirmationEmail, confirmationCode);
      pushToast("success", "確認が完了しました。ログインしてください。");
      setConfirmationCode("");
      window.location.hash = buildHash("login");
    } catch (error) {
      pushToast("error", normalizeAuthError(error));
    }
  }

  async function handleChangePassword() {
    try {
      await changePassword(currentPassword, nextPassword);
      setCurrentPassword("");
      setNextPassword("");
      pushToast("success", "パスワードを変更しました。");
    } catch (error) {
      pushToast("error", normalizeAuthError(error));
    }
  }

  async function handleDeleteAccount() {
    try {
      await deleteCurrentUser();
      setMessages([]);
      await refreshUser();
      window.location.hash = buildHash("home");
      pushToast("success", "退会処理を完了しました。");
    } catch (error) {
      pushToast("error", normalizeAuthError(error));
    }
  }

  async function handleSignOut() {
    try {
      await signOutCurrentUser();
      setMessages([]);
      await refreshUser();
      window.location.hash = buildHash("home");
      pushToast("success", "ログアウトしました。");
    } catch (error) {
      pushToast("error", normalizeAuthError(error));
    }
  }

  return (
    <SiteShell
      title={definition.title}
      showHeader={definition.showHeader}
      menuItems={menuItems}
      menuOpen={menuOpen}
      onToggleMenu={() => setMenuOpen((current) => !current)}
      actionLabel={actionLabel}
      actionHref={actionHref}
      toasts={toasts}
    >
      {!frontendConfig.region ||
      !frontendConfig.userPoolId ||
      !frontendConfig.userPoolClientId ? (
        <div className="status-card">
          Cognito関連のビルド時環境変数が未設定です。認証機能の動作確認には
          <code> NEXT_PUBLIC_AWS_REGION </code>
          <code> NEXT_PUBLIC_COGNITO_USER_POOL_ID </code>
          <code> NEXT_PUBLIC_COGNITO_USER_POOL_WEB_CLIENT_ID </code>
          の設定が必要です。
        </div>
      ) : null}
      {route === "home" ? (
        <HomeSection loginHref={buildHash("login")} termsHref={buildHash("terms")} />
      ) : null}
      {route === "login" ? (
        <LoginSection
          allowDomain={allowDomain}
          signInEmail={signInEmail}
          signInPassword={signInPassword}
          signUpEmail={signUpEmail}
          signUpPassword={signUpPassword}
          confirmationEmail={confirmationEmail}
          confirmationCode={confirmationCode}
          onSignInEmailChange={setSignInEmail}
          onSignInPasswordChange={setSignInPassword}
          onSignUpEmailChange={setSignUpEmail}
          onSignUpPasswordChange={setSignUpPassword}
          onConfirmationEmailChange={setConfirmationEmail}
          onConfirmationCodeChange={setConfirmationCode}
          onSignIn={handleSignIn}
          onSignUp={handleSignUp}
          onConfirm={handleConfirmSignUp}
        />
      ) : null}
      {route === "chat" ? (
        <ChatSection
          messages={messages}
          draft={draft}
          signedIn={signedIn}
          onDraftChange={setDraft}
          onRefresh={refreshChats}
          onSubmit={handlePostChat}
        />
      ) : null}
      {route === "mypage" ? (
        <MyPageSection
          userLabel={userLabel}
          currentPassword={currentPassword}
          nextPassword={nextPassword}
          onCurrentPasswordChange={setCurrentPassword}
          onNextPasswordChange={setNextPassword}
          onChangePassword={handleChangePassword}
          onDeleteAccount={handleDeleteAccount}
          onSignOut={handleSignOut}
        />
      ) : null}
      {route === "terms" ? <TermsSection /> : null}
    </SiteShell>
  );
}
