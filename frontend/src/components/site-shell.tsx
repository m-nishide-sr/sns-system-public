import type { ReactNode } from "react";
import type { ToastState } from "@/src/types/app";

type MenuItem = {
  href: string;
  label: string;
};

type SiteShellProps = {
  title: string;
  showHeader: boolean;
  menuItems: MenuItem[];
  menuOpen: boolean;
  onToggleMenu: () => void;
  actionLabel: string;
  actionHref: string;
  children: ReactNode;
  toasts: ToastState[];
};

export function SiteShell({
  title,
  showHeader,
  menuItems,
  menuOpen,
  onToggleMenu,
  actionLabel,
  actionHref,
  children,
  toasts,
}: SiteShellProps) {
  return (
    <div className="app-shell">
      {showHeader ? (
        <header className="app-header">
          <button
            className="icon-button"
            onClick={onToggleMenu}
            type="button"
            aria-label="メニューを開く"
          >
            ☰
          </button>
          <div className="header-title">{title}</div>
          <a className="secondary-button header-action" href={actionHref}>
            {actionLabel}
          </a>
        </header>
      ) : null}
      {menuOpen ? (
        <>
          <button
            type="button"
            className="menu-backdrop"
            aria-label="メニューを閉じる"
            onClick={onToggleMenu}
          />
          <aside className="menu-panel" aria-label="ナビゲーションメニュー">
            <button
              className="secondary-button"
              type="button"
              onClick={onToggleMenu}
            >
              閉じる
            </button>
            <nav>
              {menuItems.map((item) => (
                <a key={item.href} className="menu-link" href={item.href}>
                  <span>{item.label}</span>
                  <span>›</span>
                </a>
              ))}
            </nav>
          </aside>
        </>
      ) : null}
      <main className="page-main">{children}</main>
      <footer className="app-footer">
        © 2026 SNSシステム. All rights reserved under the Universal Copyright
        Convention.
      </footer>
      <div className="toast-viewport" aria-live="polite">
        {toasts.map((toast) => (
          <div
            key={toast.id}
            className={`toast ${
              toast.kind === "success" ? "toast-success" : "toast-error"
            }`}
          >
            {toast.message}
          </div>
        ))}
      </div>
    </div>
  );
}
