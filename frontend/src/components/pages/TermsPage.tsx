import type { AppRoute } from "@/hooks/useHashRouter";

type TermsPageProps = {
  onNavigate: (route: AppRoute) => void;
};

export function TermsPageView({ onNavigate }: TermsPageProps) {
  return (
    <div className="mx-auto max-w-4xl rounded-[2rem] border border-slate-200 bg-white p-6 shadow-xl sm:p-10">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-sm font-semibold text-blue-600">Terms of Service</p>
          <h1 className="mt-2 text-3xl font-bold text-slate-900">利用規約</h1>
        </div>
        <button
          className="inline-flex items-center justify-center rounded-full border border-slate-300 px-5 py-3 text-sm font-semibold text-slate-700 transition hover:border-blue-300 hover:text-blue-700"
          onClick={() => onNavigate("/")}
          type="button"
        >
          トップへ戻る
        </button>
      </div>
      <div className="mt-8 space-y-8 text-sm leading-7 text-slate-600">
        <section>
          <h2 className="text-lg font-semibold text-slate-900">1. 目的</h2>
          <p className="mt-2">本サービスは、SNSシステム利用者間の業務上のコミュニケーションを円滑にすることを目的として提供されます。</p>
        </section>
        <section>
          <h2 className="text-lg font-semibold text-slate-900">2. アカウント管理</h2>
          <p className="mt-2">利用者は、自身の認証情報を適切に管理し、第三者へ共有してはなりません。不正利用が判明した場合は速やかに管理者へ連絡してください。</p>
        </section>
        <section>
          <h2 className="text-lg font-semibold text-slate-900">3. 投稿内容</h2>
          <p className="mt-2">法令や公序良俗に反する内容、誹謗中傷、機密情報の無断掲載は禁止します。投稿内容は送信後に編集も削除もできません。内容を確認したうえで投稿してください。</p>
        </section>
        <section>
          <h2 className="text-lg font-semibold text-slate-900">4. サービス変更</h2>
          <p className="mt-2">運営者は、保守や改善のために本サービスの全部または一部を変更、停止、終了することがあります。</p>
        </section>
        <section>
          <h2 className="text-lg font-semibold text-slate-900">5. 免責</h2>
          <p className="mt-2">運営者は、本サービスの利用により生じた損害について、故意または重過失がある場合を除き責任を負いません。</p>
        </section>
      </div>
    </div>
  );
}

export function TermsPage(props: TermsPageProps) {
  return <TermsPageView {...props} />;
}
