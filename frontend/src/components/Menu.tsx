/**
 * ハンバーガーメニューから表示するサイドメニュー。
 */
import type { AppRoute } from "@/hooks/useHashRouter";

type MenuViewProps = {
  currentRoute: AppRoute;
  isAuthenticated: boolean;
  isOpen: boolean;
  onClose: () => void;
  onNavigate: (route: AppRoute) => void;
};

const MENU_ITEMS: Array<{ route: AppRoute; label: string }> = [
  { route: "/", label: "トップ" },
  { route: "/chat", label: "チャット" },
  { route: "/login", label: "ログイン / 新規登録" },
  { route: "/mypage", label: "マイページ" },
  { route: "/terms", label: "利用規約" },
];

/** サイドメニューの見た目を描画する Presentational Component。 */
export function MenuView({ currentRoute, isAuthenticated, isOpen, onClose, onNavigate }: MenuViewProps) {
  return (
    <>
      <button
        aria-hidden={!isOpen}
        aria-label="メニューを閉じる"
        className={`fixed inset-0 z-40 bg-slate-950/40 transition ${isOpen ? "pointer-events-auto opacity-100" : "pointer-events-none opacity-0"}`}
        onClick={onClose}
        type="button"
      />
      <aside
        aria-hidden={!isOpen}
        className={`fixed inset-y-0 left-0 z-50 flex w-80 max-w-[85vw] flex-col border-r border-slate-200 bg-white shadow-2xl transition-transform ${isOpen ? "translate-x-0" : "-translate-x-full"}`}
      >
        <div className="flex items-center justify-between border-b border-slate-200 px-5 py-4">
          <div>
            <p className="text-sm font-medium text-blue-600">SNSシステム</p>
            <h2 className="text-lg font-semibold text-slate-900">メニュー</h2>
          </div>
          <button
            aria-label="メニューを閉じる"
            className="inline-flex h-10 w-10 items-center justify-center rounded-full border border-slate-200 text-slate-600 transition hover:bg-slate-100"
            onClick={onClose}
            type="button"
          >
            ✕
          </button>
        </div>
        <nav className="flex-1 space-y-2 px-3 py-4">
          {MENU_ITEMS.map((item) => {
            const isDisabled = item.route === "/mypage" && !isAuthenticated;
            const isCurrent = currentRoute === item.route;

            return (
              <button
                key={item.route}
                className={`flex w-full items-center justify-between rounded-2xl px-4 py-3 text-left text-sm font-medium transition ${isCurrent ? "bg-blue-600 text-white shadow-lg shadow-blue-600/20" : "text-slate-700 hover:bg-slate-100"} ${isDisabled ? "cursor-not-allowed opacity-50" : ""}`}
                disabled={isDisabled}
                onClick={() => onNavigate(item.route)}
                type="button"
              >
                <span>{item.label}</span>
                {isCurrent ? <span className="text-xs">現在地</span> : null}
              </button>
            );
          })}
        </nav>
        <div className="border-t border-slate-200 px-5 py-4 text-sm text-slate-500">
          {isAuthenticated ? "ログイン中です。チャットやアカウント設定を利用できます。" : "未ログインです。ログインするとチャットとマイページを利用できます。"}
        </div>
      </aside>
    </>
  );
}

/** メニュー描画をラップする軽量 Container Component。 */
export function Menu(props: MenuViewProps) {
  return <MenuView {...props} />;
}
