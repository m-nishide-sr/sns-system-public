import { useState } from "react";
import { deleteUser, signOut, updatePassword } from "aws-amplify/auth";
import type { AppRoute } from "@/hooks/useHashRouter";
import type { ToastTone } from "@/components/Toast";

type MyPageProps = {
  isAuthenticated: boolean;
  isAuthChecking: boolean;
  onAuthChange: () => Promise<boolean> | Promise<void> | boolean | void;
  onNavigate: (route: AppRoute) => void;
  onToast: (message: string, tone?: ToastTone) => void;
};

type MyPageViewProps = {
  isAuthenticated: boolean;
  isAuthChecking: boolean;
  oldPassword: string;
  newPassword: string;
  deleteConfirmation: string;
  isBusy: boolean;
  onChangeField: (field: "oldPassword" | "newPassword" | "deleteConfirmation", value: string) => void;
  onChangePassword: () => void;
  onDeleteAccount: () => void;
  onLogout: () => void;
  onNavigate: (route: AppRoute) => void;
};

export function MyPageView({
  isAuthenticated,
  isAuthChecking,
  oldPassword,
  newPassword,
  deleteConfirmation,
  isBusy,
  onChangeField,
  onChangePassword,
  onDeleteAccount,
  onLogout,
  onNavigate,
}: MyPageViewProps) {
  if (isAuthChecking) {
    return <div className="mx-auto max-w-4xl rounded-3xl bg-white p-10 text-center shadow-lg">認証状態を確認しています...</div>;
  }

  if (!isAuthenticated) {
    return (
      <div className="mx-auto max-w-3xl rounded-[2rem] border border-slate-200 bg-white p-8 text-center shadow-xl">
        <h2 className="text-2xl font-semibold text-slate-900">ログインが必要です</h2>
        <p className="mt-3 text-slate-600">マイページを利用するにはサインインしてください。</p>
        <button
          className="mt-6 inline-flex rounded-full bg-slate-900 px-6 py-3 text-sm font-semibold text-white transition hover:bg-blue-600"
          onClick={() => onNavigate("/login")}
          type="button"
        >
          ログインページへ
        </button>
      </div>
    );
  }

  return (
    <div className="mx-auto grid max-w-5xl gap-6 lg:grid-cols-2">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-xl sm:p-8">
        <h2 className="text-2xl font-semibold text-slate-900">パスワード変更</h2>
        <p className="mt-2 text-sm leading-6 text-slate-600">現在のパスワードを確認したうえで、新しいパスワードに更新します。</p>
        <div className="mt-6 space-y-4">
          <label className="block">
            <span className="mb-2 block text-sm font-medium text-slate-700">現在のパスワード</span>
            <input
              className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
              onChange={(event) => onChangeField("oldPassword", event.target.value)}
              type="password"
              value={oldPassword}
            />
          </label>
          <label className="block">
            <span className="mb-2 block text-sm font-medium text-slate-700">新しいパスワード</span>
            <input
              className="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
              onChange={(event) => onChangeField("newPassword", event.target.value)}
              type="password"
              value={newPassword}
            />
          </label>
          <button
            className="inline-flex rounded-full bg-slate-900 px-5 py-3 text-sm font-semibold text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
            disabled={isBusy}
            onClick={onChangePassword}
            type="button"
          >
            {isBusy ? "更新中..." : "パスワードを変更"}
          </button>
        </div>
      </section>
      <section className="space-y-6">
        <div className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-xl sm:p-8">
          <h2 className="text-2xl font-semibold text-slate-900">ログアウト</h2>
          <p className="mt-2 text-sm leading-6 text-slate-600">現在のセッションを終了し、ログインページへ戻ります。</p>
          <button
            className="mt-6 inline-flex rounded-full border border-slate-300 px-5 py-3 text-sm font-semibold text-slate-700 transition hover:border-blue-300 hover:text-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
            disabled={isBusy}
            onClick={onLogout}
            type="button"
          >
            ログアウト
          </button>
        </div>
        <div className="rounded-[2rem] border border-rose-200 bg-rose-50 p-6 shadow-xl sm:p-8">
          <h2 className="text-2xl font-semibold text-rose-900">退会</h2>
          <p className="mt-2 text-sm leading-6 text-rose-800">
            退会すると認証アカウントを削除します。投稿済みデータは削除されません。実行する場合は <span className="font-semibold">DELETE</span> と入力してください。
          </p>
          <input
            className="mt-4 w-full rounded-2xl border border-rose-200 bg-white px-4 py-3 outline-none transition focus:border-rose-400 focus:ring-4 focus:ring-rose-100"
            onChange={(event) => onChangeField("deleteConfirmation", event.target.value)}
            value={deleteConfirmation}
          />
          <button
            className="mt-4 inline-flex rounded-full bg-rose-600 px-5 py-3 text-sm font-semibold text-white transition hover:bg-rose-700 disabled:cursor-not-allowed disabled:opacity-50"
            disabled={isBusy || deleteConfirmation !== "DELETE"}
            onClick={onDeleteAccount}
            type="button"
          >
            アカウントを削除する
          </button>
        </div>
      </section>
    </div>
  );
}

export function MyPage({ isAuthenticated, isAuthChecking, onAuthChange, onNavigate, onToast }: MyPageProps) {
  const [oldPassword, setOldPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [deleteConfirmation, setDeleteConfirmation] = useState("");
  const [isBusy, setIsBusy] = useState(false);

  const setBusy = async (callback: () => Promise<void>) => {
    setIsBusy(true);
    try {
      await callback();
    } catch (error) {
      const message = error instanceof Error ? error.message : "アカウント操作に失敗しました。";
      onToast(message, "error");
    } finally {
      setIsBusy(false);
    }
  };

  const handleChangeField = (field: "oldPassword" | "newPassword" | "deleteConfirmation", value: string) => {
    if (field === "oldPassword") {
      setOldPassword(value);
      return;
    }
    if (field === "newPassword") {
      setNewPassword(value);
      return;
    }
    setDeleteConfirmation(value);
  };

  const handleChangePassword = () => void setBusy(async () => {
    if (!oldPassword || !newPassword) {
      throw new Error("現在のパスワードと新しいパスワードを入力してください。");
    }

    await updatePassword({ oldPassword, newPassword });
    setOldPassword("");
    setNewPassword("");
    onToast("パスワードを変更しました。", "success");
  });

  const handleLogout = () => void setBusy(async () => {
    await signOut();
    await onAuthChange();
    onToast("ログアウトしました。", "success");
    onNavigate("/login");
  });

  const handleDeleteAccount = () => void setBusy(async () => {
    if (deleteConfirmation !== "DELETE") {
      throw new Error("DELETE と入力すると退会できます。");
    }

    await deleteUser();
    await onAuthChange();
    setDeleteConfirmation("");
    onToast("アカウントを削除しました。", "success");
    onNavigate("/");
  });

  return (
    <MyPageView
      deleteConfirmation={deleteConfirmation}
      isAuthenticated={isAuthenticated}
      isAuthChecking={isAuthChecking}
      isBusy={isBusy}
      newPassword={newPassword}
      oldPassword={oldPassword}
      onChangeField={handleChangeField}
      onChangePassword={handleChangePassword}
      onDeleteAccount={handleDeleteAccount}
      onLogout={handleLogout}
      onNavigate={onNavigate}
    />
  );
}
