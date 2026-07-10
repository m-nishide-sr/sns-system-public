type HeaderViewProps = {
  isAuthenticated: boolean;
  title: string;
  onMenuToggle: () => void;
  onMyPageClick: () => void;
  onLoginClick: () => void;
};

export function HeaderView({
  isAuthenticated,
  title,
  onMenuToggle,
  onMyPageClick,
  onLoginClick,
}: HeaderViewProps) {
  return (
    <header className="fixed inset-x-0 top-0 z-40 border-b border-slate-200/80 bg-white/90 backdrop-blur">
      <div className="mx-auto flex h-16 max-w-6xl items-center gap-3 px-4 sm:px-6 lg:px-8">
        <button
          aria-label="メニューを開く"
          className="inline-flex h-10 w-10 items-center justify-center rounded-full border border-slate-200 text-slate-700 transition hover:border-blue-300 hover:bg-blue-50"
          onClick={onMenuToggle}
          type="button"
        >
          <span className="text-xl leading-none">☰</span>
        </button>
        <div className="min-w-0 flex-1 text-center text-base font-semibold text-slate-900 sm:text-lg">
          <span className="block truncate">{title}</span>
        </div>
        <button
          className="inline-flex min-w-20 items-center justify-center rounded-full bg-slate-900 px-4 py-2 text-sm font-semibold text-white transition hover:bg-blue-600"
          onClick={isAuthenticated ? onMyPageClick : onLoginClick}
          type="button"
        >
          {isAuthenticated ? "マイページ" : "ログイン"}
        </button>
      </div>
    </header>
  );
}

export function Header(props: HeaderViewProps) {
  return <HeaderView {...props} />;
}
