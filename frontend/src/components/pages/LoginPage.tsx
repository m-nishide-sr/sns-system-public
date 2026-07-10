import { useMemo, useState } from "react";
import {
  confirmResetPassword,
  confirmSignUp,
  resetPassword,
  signIn,
  signUp,
} from "aws-amplify/auth";
import type { AppRoute } from "@/hooks/useHashRouter";
import type { ToastTone } from "@/components/Toast";

type AuthTab = "login" | "register";
type FlowStep = "auth" | "confirm-signup" | "request-reset" | "confirm-reset";
type FieldName =
  | "loginEmail"
  | "loginPassword"
  | "registerEmail"
  | "registerPassword"
  | "confirmCode"
  | "resetEmail"
  | "resetCode"
  | "resetNewPassword";

type LoginPageProps = {
  isAuthenticated: boolean;
  onAuthSuccess: () => Promise<void> | void;
  onNavigate: (route: AppRoute) => void;
  onToast: (message: string, tone?: ToastTone) => void;
};

type LoginPageViewProps = {
  activeTab: AuthTab;
  flowStep: FlowStep;
  loginEmail: string;
  loginPassword: string;
  registerEmail: string;
  registerPassword: string;
  confirmCode: string;
  resetEmail: string;
  resetCode: string;
  resetNewPassword: string;
  isBusy: boolean;
  confirmationEmail: string;
  isAuthenticated: boolean;
  onChangeField: (field: FieldName, value: string) => void;
  onChangeTab: (tab: AuthTab) => void;
  onStartLogin: () => void;
  onStartRegister: () => void;
  onStartReset: () => void;
  onConfirmSignUp: () => void;
  onRequestReset: () => void;
  onConfirmReset: () => void;
  onBackToAuth: () => void;
  onNavigate: (route: AppRoute) => void;
};

const ALLOW_DOMAIN_NOTE =
  "新規登録は管理者が許可したメールドメインに限られます。";

export function LoginPageView({
  activeTab,
  flowStep,
  loginEmail,
  loginPassword,
  registerEmail,
  registerPassword,
  confirmCode,
  resetEmail,
  resetCode,
  resetNewPassword,
  isBusy,
  confirmationEmail,
  isAuthenticated,
  onChangeField,
  onChangeTab,
  onStartLogin,
  onStartRegister,
  onStartReset,
  onConfirmSignUp,
  onRequestReset,
  onConfirmReset,
  onBackToAuth,
  onNavigate,
}: LoginPageViewProps) {
  return (
    <div className="mx-auto grid max-w-6xl gap-6 lg:grid-cols-[0.9fr_1.1fr]">
      <section className="rounded-[2rem] border border-slate-200 bg-slate-900 p-8 text-white shadow-2xl shadow-slate-900/15">
        <span className="inline-flex rounded-full bg-white/10 px-3 py-1 text-sm font-medium text-blue-100">
          認証
        </span>
        <h1 className="mt-4 text-3xl font-bold">ログイン / 新規登録</h1>
        <p className="mt-4 text-sm leading-7 text-slate-200">
          Cognitoを利用してログイン、新規登録、メール確認、パスワード再設定を行えます。確認コードはメールで送信されます。
        </p>
        <div className="mt-6 rounded-3xl bg-white/10 p-5 text-sm leading-7 text-slate-100">
          <p className="font-semibold text-white">登録時の注意</p>
          <p className="mt-2">{ALLOW_DOMAIN_NOTE}</p>
          <p className="mt-2">
            登録直後または未確認状態のアカウントでは、確認コードの入力が必要です。
          </p>
        </div>
        <div className="mt-6 space-y-3 text-sm text-slate-200">
          <button
            className="block font-semibold text-white underline-offset-4 hover:underline"
            onClick={() => onNavigate("/terms")}
            type="button"
          >
            利用規約を確認する
          </button>
          {isAuthenticated ? (
            <button
              className="block font-semibold text-white underline-offset-4 hover:underline"
              onClick={() => onNavigate("/chat")}
              type="button"
            >
              ログイン済みのためチャットへ移動する
            </button>
          ) : null}
        </div>
      </section>
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-xl sm:p-8">
        {flowStep === "auth" ? (
          <>
            <div className="flex rounded-full bg-slate-100 p-1">
              <button
                className={`flex-1 rounded-full px-4 py-2 text-sm font-semibold transition ${activeTab === "login" ? "bg-white text-slate-900 shadow" : "text-slate-500"}`}
                onClick={() => onChangeTab("login")}
                type="button"
              >
                ログイン
              </button>
              <button
                className={`flex-1 rounded-full px-4 py-2 text-sm font-semibold transition ${activeTab === "register" ? "bg-white text-slate-900 shadow" : "text-slate-500"}`}
                onClick={() => onChangeTab("register")}
                type="button"
              >
                新規登録
              </button>
            </div>
            {activeTab === "login" ? (
              <div className="mt-6 space-y-4">
                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-slate-700">
                    メールアドレス
                  </span>
                  <input
                    className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
                    onChange={(event) =>
                      onChangeField("loginEmail", event.target.value)
                    }
                    placeholder="you@example.com"
                    type="email"
                    value={loginEmail}
                  />
                </label>
                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-slate-700">
                    パスワード
                  </span>
                  <input
                    className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
                    onChange={(event) =>
                      onChangeField("loginPassword", event.target.value)
                    }
                    type="password"
                    value={loginPassword}
                  />
                </label>
                <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                  <button
                    className="inline-flex items-center justify-center rounded-full bg-slate-900 px-5 py-3 text-sm font-semibold text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={isBusy}
                    onClick={onStartLogin}
                    type="button"
                  >
                    {isBusy ? "処理中..." : "ログイン"}
                  </button>
                  <button
                    className="text-sm font-semibold text-blue-700 underline-offset-4 hover:underline"
                    onClick={onStartReset}
                    type="button"
                  >
                    パスワードを忘れた場合
                  </button>
                </div>
              </div>
            ) : (
              <div className="mt-6 space-y-4">
                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-slate-700">
                    メールアドレス
                  </span>
                  <input
                    className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
                    onChange={(event) =>
                      onChangeField("registerEmail", event.target.value)
                    }
                    placeholder="allowed-domain@example.com"
                    type="email"
                    value={registerEmail}
                  />
                </label>
                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-slate-700">
                    パスワード
                  </span>
                  <input
                    className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
                    onChange={(event) =>
                      onChangeField("registerPassword", event.target.value)
                    }
                    type="password"
                    value={registerPassword}
                  />
                </label>
                <p className="rounded-2xl bg-amber-50 px-4 py-3 text-sm leading-6 text-amber-800">
                  {ALLOW_DOMAIN_NOTE}
                </p>
                <button
                  className="inline-flex items-center justify-center rounded-full bg-slate-900 px-5 py-3 text-sm font-semibold text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
                  disabled={isBusy}
                  onClick={onStartRegister}
                  type="button"
                >
                  {isBusy ? "処理中..." : "新規登録"}
                </button>
              </div>
            )}
          </>
        ) : null}

        {flowStep === "confirm-signup" ? (
          <div className="space-y-5">
            <div>
              <h2 className="text-2xl font-semibold text-slate-900">
                確認コードの入力
              </h2>
              <p className="mt-2 text-sm leading-6 text-slate-600">
                {confirmationEmail || "対象メールアドレス"}{" "}
                に届いた確認コードを入力してください。
              </p>
            </div>
            <label className="block">
              <span className="mb-2 block text-sm font-medium text-slate-700">
                確認コード
              </span>
              <input
                className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
                onChange={(event) =>
                  onChangeField("confirmCode", event.target.value)
                }
                value={confirmCode}
              />
            </label>
            <div className="flex flex-col gap-3 sm:flex-row">
              <button
                className="inline-flex items-center justify-center rounded-full bg-slate-900 px-5 py-3 text-sm font-semibold text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
                disabled={isBusy}
                onClick={onConfirmSignUp}
                type="button"
              >
                {isBusy ? "確認中..." : "確認して続行"}
              </button>
              <button
                className="inline-flex items-center justify-center rounded-full border border-slate-300 px-5 py-3 text-sm font-semibold text-slate-700 transition hover:border-blue-300 hover:text-blue-700"
                onClick={onBackToAuth}
                type="button"
              >
                戻る
              </button>
            </div>
          </div>
        ) : null}

        {flowStep === "request-reset" ? (
          <div className="space-y-5">
            <div>
              <h2 className="text-2xl font-semibold text-slate-900">
                パスワード再設定
              </h2>
              <p className="mt-2 text-sm leading-6 text-slate-600">
                登録済みメールアドレスへ確認コードを送信します。
              </p>
            </div>
            <label className="block">
              <span className="mb-2 block text-sm font-medium text-slate-700">
                メールアドレス
              </span>
              <input
                className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
                onChange={(event) =>
                  onChangeField("resetEmail", event.target.value)
                }
                type="email"
                value={resetEmail}
              />
            </label>
            <div className="flex flex-col gap-3 sm:flex-row">
              <button
                className="inline-flex items-center justify-center rounded-full bg-slate-900 px-5 py-3 text-sm font-semibold text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
                disabled={isBusy}
                onClick={onRequestReset}
                type="button"
              >
                {isBusy ? "送信中..." : "確認コードを送信"}
              </button>
              <button
                className="inline-flex items-center justify-center rounded-full border border-slate-300 px-5 py-3 text-sm font-semibold text-slate-700 transition hover:border-blue-300 hover:text-blue-700"
                onClick={onBackToAuth}
                type="button"
              >
                ログインへ戻る
              </button>
            </div>
          </div>
        ) : null}

        {flowStep === "confirm-reset" ? (
          <div className="space-y-5">
            <div>
              <h2 className="text-2xl font-semibold text-slate-900">
                新しいパスワードを設定
              </h2>
              <p className="mt-2 text-sm leading-6 text-slate-600">
                メールで受け取った確認コードと新しいパスワードを入力してください。
              </p>
            </div>
            <label className="block">
              <span className="mb-2 block text-sm font-medium text-slate-700">
                確認コード
              </span>
              <input
                className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
                onChange={(event) =>
                  onChangeField("resetCode", event.target.value)
                }
                value={resetCode}
              />
            </label>
            <label className="block">
              <span className="mb-2 block text-sm font-medium text-slate-700">
                新しいパスワード
              </span>
              <input
                className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
                onChange={(event) =>
                  onChangeField("resetNewPassword", event.target.value)
                }
                type="password"
                value={resetNewPassword}
              />
            </label>
            <div className="flex flex-col gap-3 sm:flex-row">
              <button
                className="inline-flex items-center justify-center rounded-full bg-slate-900 px-5 py-3 text-sm font-semibold text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
                disabled={isBusy}
                onClick={onConfirmReset}
                type="button"
              >
                {isBusy ? "更新中..." : "パスワードを更新"}
              </button>
              <button
                className="inline-flex items-center justify-center rounded-full border border-slate-300 px-5 py-3 text-sm font-semibold text-slate-700 transition hover:border-blue-300 hover:text-blue-700"
                onClick={onBackToAuth}
                type="button"
              >
                ログインへ戻る
              </button>
            </div>
          </div>
        ) : null}
      </section>
    </div>
  );
}

export function LoginPage({
  isAuthenticated,
  onAuthSuccess,
  onNavigate,
  onToast,
}: LoginPageProps) {
  const [activeTab, setActiveTab] = useState<AuthTab>("login");
  const [flowStep, setFlowStep] = useState<FlowStep>("auth");
  const [isBusy, setIsBusy] = useState(false);
  const [fields, setFields] = useState<Record<FieldName, string>>({
    loginEmail: "",
    loginPassword: "",
    registerEmail: "",
    registerPassword: "",
    confirmCode: "",
    resetEmail: "",
    resetCode: "",
    resetNewPassword: "",
  });
  const [confirmationEmail, setConfirmationEmail] = useState("");

  const changeField = (field: FieldName, value: string) => {
    setFields((current) => ({ ...current, [field]: value }));
  };

  const setBusy = async (callback: () => Promise<void>) => {
    setIsBusy(true);
    try {
      await callback();
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "認証処理に失敗しました。";
      onToast(message, "error");
    } finally {
      setIsBusy(false);
    }
  };

  const confirmationTarget = useMemo(
    () => confirmationEmail || fields.registerEmail || fields.loginEmail,
    [confirmationEmail, fields.loginEmail, fields.registerEmail],
  );

  const handleBack = () => {
    setFlowStep("auth");
  };

  const handleStartReset = () => {
    setFields((current) => ({
      ...current,
      resetEmail: current.resetEmail || current.loginEmail,
    }));
    setFlowStep("request-reset");
  };

  const handleLogin = () =>
    void setBusy(async () => {
      const username = fields.loginEmail.trim();
      const password = fields.loginPassword;
      if (!username || !password) {
        throw new Error("メールアドレスとパスワードを入力してください。");
      }

      const result = await signIn({ username, password });
      if (result.nextStep.signInStep === "CONFIRM_SIGN_UP") {
        setConfirmationEmail(username);
        setFlowStep("confirm-signup");
        onToast(
          "メール確認が必要です。確認コードを入力してください。",
          "success",
        );
        return;
      }

      onToast("ログインしました。", "success");
      await onAuthSuccess();
    });

  const handleRegister = () =>
    void setBusy(async () => {
      const email = fields.registerEmail.trim();
      const password = fields.registerPassword;
      if (!email || !password) {
        throw new Error("メールアドレスとパスワードを入力してください。");
      }

      const result = await signUp({
        username: email,
        password,
        options: {
          userAttributes: {
            email,
          },
        },
      });

      setConfirmationEmail(email);
      if (result.nextStep.signUpStep === "DONE") {
        onToast("登録が完了しました。ログインしてください。", "success");
        setActiveTab("login");
        return;
      }

      setFlowStep("confirm-signup");
      onToast(
        "確認コードを送信しました。メールを確認してください。",
        "success",
      );
    });

  const handleConfirmSignUp = () =>
    void setBusy(async () => {
      const username = confirmationTarget.trim();
      const confirmationCode = fields.confirmCode.trim();
      if (!username || !confirmationCode) {
        throw new Error("メールアドレスと確認コードを入力してください。");
      }

      await confirmSignUp({ username, confirmationCode });
      setFlowStep("auth");
      setActiveTab("login");
      setFields((current) => ({
        ...current,
        confirmCode: "",
        loginEmail: username,
      }));
      onToast("メール確認が完了しました。ログインしてください。", "success");
    });

  const handleRequestReset = () =>
    void setBusy(async () => {
      const username = fields.resetEmail.trim();
      if (!username) {
        throw new Error("メールアドレスを入力してください。");
      }

      await resetPassword({ username });
      setFlowStep("confirm-reset");
      onToast(
        "確認コードを送信しました。メールを確認してください。",
        "success",
      );
    });

  const handleConfirmReset = () =>
    void setBusy(async () => {
      const username = fields.resetEmail.trim();
      const confirmationCode = fields.resetCode.trim();
      const newPassword = fields.resetNewPassword;
      if (!username || !confirmationCode || !newPassword) {
        throw new Error(
          "メールアドレス、確認コード、新しいパスワードを入力してください。",
        );
      }

      await confirmResetPassword({ username, confirmationCode, newPassword });
      setFlowStep("auth");
      setActiveTab("login");
      setFields((current) => ({
        ...current,
        loginEmail: username,
        loginPassword: "",
        resetCode: "",
        resetNewPassword: "",
      }));
      onToast(
        "パスワードを更新しました。新しいパスワードでログインしてください。",
        "success",
      );
    });

  return (
    <LoginPageView
      activeTab={activeTab}
      confirmationEmail={confirmationTarget}
      confirmCode={fields.confirmCode}
      flowStep={flowStep}
      isAuthenticated={isAuthenticated}
      isBusy={isBusy}
      loginEmail={fields.loginEmail}
      loginPassword={fields.loginPassword}
      onBackToAuth={handleBack}
      onChangeField={changeField}
      onChangeTab={setActiveTab}
      onConfirmReset={handleConfirmReset}
      onConfirmSignUp={handleConfirmSignUp}
      onNavigate={onNavigate}
      onRequestReset={handleRequestReset}
      onStartLogin={handleLogin}
      onStartRegister={handleRegister}
      onStartReset={handleStartReset}
      registerEmail={fields.registerEmail}
      registerPassword={fields.registerPassword}
      resetCode={fields.resetCode}
      resetEmail={fields.resetEmail}
      resetNewPassword={fields.resetNewPassword}
    />
  );
}
