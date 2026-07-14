/**
 * サービス概要と主要導線を表示するトップページ。
 */
import type { AppRoute } from "@/hooks/useHashRouter";

type TopPageViewProps = {
  isAuthenticated: boolean;
  onNavigate: (route: AppRoute) => void;
};

/** トップページの見た目だけを描画する Presentational Component。 */
export function TopPageView({ isAuthenticated, onNavigate }: TopPageViewProps) {
  return (
    <div className="mx-auto flex min-h-[calc(100vh-8rem)] max-w-6xl items-center">
      <div className="grid w-full gap-10 lg:grid-cols-[1.2fr_0.8fr] lg:items-center">
        <section className="space-y-6">
          <span className="inline-flex rounded-full bg-blue-100 px-4 py-1 text-sm font-semibold text-blue-700">
            社内SNS / 静的SPA
          </span>
          <div className="space-y-4">
            <h1 className="text-4xl font-bold tracking-tight text-slate-900 sm:text-5xl">
              チームのやり取りを、
              <br />
              もっとシンプルに。
            </h1>
            <p className="max-w-2xl text-lg leading-8 text-slate-600">
              SNSシステムは、チーム内の短い共有や気づきをスムーズに投稿できる軽量なコミュニケーション基盤です。ログインすると、タイムラインの閲覧と投稿、アカウント管理をすぐに利用できます。
            </p>
          </div>
          <div className="flex flex-col gap-3 sm:flex-row">
            <button
              className="inline-flex items-center justify-center rounded-full bg-slate-900 px-6 py-3 text-sm font-semibold text-white transition hover:bg-blue-600"
              onClick={() => onNavigate(isAuthenticated ? "/chat" : "/login")}
              type="button"
            >
              {isAuthenticated ? "チャットを開く" : "ログインして始める"}
            </button>
            <button
              className="inline-flex items-center justify-center rounded-full border border-slate-300 bg-white px-6 py-3 text-sm font-semibold text-slate-700 transition hover:border-blue-300 hover:text-blue-700"
              onClick={() => onNavigate("/terms")}
              type="button"
            >
              利用規約を見る
            </button>
          </div>
        </section>
        <section className="rounded-[2rem] border border-white/70 bg-white/80 p-6 shadow-2xl shadow-blue-100/50 backdrop-blur sm:p-8">
          <div className="space-y-4">
            <div className="rounded-2xl bg-slate-900 p-5 text-white">
              <p className="text-sm text-slate-300">特徴</p>
              <ul className="mt-3 space-y-2 text-sm leading-6 text-slate-100">
                <li>・同一ドメイン配下のAPIと安全に連携</li>
                <li>・Cognito認証によるサインイン / 退会 / パスワード変更</li>
                <li>・モバイルでも見やすいレスポンシブUI</li>
              </ul>
            </div>
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="rounded-2xl border border-slate-200 p-4">
                <p className="text-sm font-semibold text-slate-900">チャット</p>
                <p className="mt-2 text-sm leading-6 text-slate-600">新着確認、投稿、再読み込みをひとつの画面で完結できます。</p>
              </div>
              <div className="rounded-2xl border border-slate-200 p-4">
                <p className="text-sm font-semibold text-slate-900">アカウント</p>
                <p className="mt-2 text-sm leading-6 text-slate-600">メール確認、パスワード再設定、退会処理までフロントエンドから操作できます。</p>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}

/** トップページ描画をラップする軽量 Container Component。 */
export function TopPage(props: TopPageViewProps) {
  return <TopPageView {...props} />;
}
