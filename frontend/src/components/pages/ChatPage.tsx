/**
 * タイムライン取得と投稿を担当するチャット画面。
 */
import { useCallback, useEffect, useMemo, useState } from "react";
import type { AppRoute } from "@/hooks/useHashRouter";
import { fetchTimeline, postMessage, type TimelineMessage } from "@/lib/api";
import type { ToastTone } from "@/components/Toast";

type ChatPageProps = {
  isAuthenticated: boolean;
  isAuthChecking: boolean;
  onNavigate: (route: AppRoute) => void;
  onToast: (message: string, tone?: ToastTone) => void;
};

type ChatPageViewProps = {
  isAuthenticated: boolean;
  isAuthChecking: boolean;
  isLoading: boolean;
  isPosting: boolean;
  messages: TimelineMessage[];
  draft: string;
  hasMore: boolean;
  onChangeDraft: (value: string) => void;
  onRefresh: () => void;
  onSubmit: () => void;
  onLoadMore: () => void;
  onNavigate: (route: AppRoute) => void;
};

/** API の ISO8601 文字列を日本語表示向け日時へ整形する。 */
function formatDate(value: string) {
  return new Intl.DateTimeFormat("ja-JP", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

/** チャット画面の見た目だけを描画する Presentational Component。 */
export function ChatPageView({
  isAuthenticated,
  isAuthChecking,
  isLoading,
  isPosting,
  messages,
  draft,
  hasMore,
  onChangeDraft,
  onRefresh,
  onSubmit,
  onLoadMore,
  onNavigate,
}: ChatPageViewProps) {
  if (isAuthChecking) {
    return <div className="mx-auto max-w-4xl rounded-3xl bg-white p-10 text-center shadow-lg">認証状態を確認しています...</div>;
  }

  if (!isAuthenticated) {
    return (
      <div className="mx-auto max-w-3xl rounded-[2rem] border border-slate-200 bg-white p-8 text-center shadow-xl">
        <h2 className="text-2xl font-semibold text-slate-900">ログインが必要です</h2>
        <p className="mt-3 text-slate-600">チャットの閲覧と投稿にはサインインしてください。</p>
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
    <div className="mx-auto grid max-w-6xl gap-6 lg:grid-cols-[1.1fr_0.9fr]">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-xl sm:p-6">
        <div className="flex flex-col gap-3 border-b border-slate-200 pb-5 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-xl font-semibold text-slate-900">タイムライン</h2>
            <p className="mt-1 text-sm text-slate-500">最新50件を表示します。必要に応じて再読み込みしてください。</p>
          </div>
          <button
            className="inline-flex items-center justify-center rounded-full border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 transition hover:border-blue-300 hover:text-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
            disabled={isLoading}
            onClick={onRefresh}
            type="button"
          >
            {isLoading ? "更新中..." : "更新"}
          </button>
        </div>
        <div className="mt-5 space-y-4">
          {messages.length === 0 ? (
            <div className="rounded-2xl border border-dashed border-slate-300 px-4 py-10 text-center text-sm text-slate-500">
              まだ投稿がありません。最初のメッセージを送ってみましょう。
            </div>
          ) : (
            messages.map((message, index) => (
              <article
                key={`${message.created_at}-${message.user_name}-${index}`}
                className={`rounded-3xl border p-4 shadow-sm ${message.is_from_user ? "border-blue-100 bg-blue-50/80" : "border-slate-200 bg-slate-50"}`}
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <p className="font-semibold text-slate-900">{message.user_name}</p>
                    <p className="text-xs text-slate-500">{formatDate(message.created_at)}</p>
                  </div>
                  <span className={`rounded-full px-3 py-1 text-xs font-semibold ${message.is_from_user ? "bg-blue-600 text-white" : "bg-slate-200 text-slate-700"}`}>
                    {message.is_from_user ? "あなた" : "メンバー"}
                  </span>
                </div>
                <p className="mt-3 whitespace-pre-wrap break-words text-sm leading-7 text-slate-700">{message.body}</p>
              </article>
            ))
          )}
        </div>
        {messages.length > 0 ? (
          <div className="mt-5 flex justify-center">
            <button
              className="inline-flex items-center justify-center rounded-full border border-slate-300 px-5 py-2 text-sm font-semibold text-slate-700 transition hover:border-blue-300 hover:text-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={!hasMore || isLoading}
              onClick={onLoadMore}
              type="button"
            >
              {hasMore ? "過去の投稿をさらに読み込む" : "これ以上古い投稿はありません"}
            </button>
          </div>
        ) : null}
      </section>
      <section className="space-y-6">
        <div className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-xl">
          <div className="space-y-2">
            <h2 className="text-xl font-semibold text-slate-900">投稿する</h2>
            <p className="text-sm text-slate-500">投稿内容は公開後に編集・削除できません。送信前に内容を確認してください。</p>
          </div>
          <textarea
            className="mt-4 min-h-48 w-full rounded-3xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm leading-7 text-slate-900 outline-none transition focus:border-blue-400 focus:bg-white focus:ring-4 focus:ring-blue-100"
            onChange={(event) => onChangeDraft(event.target.value)}
            placeholder="今共有したいことを書いてください"
            value={draft}
          />
          <div className="mt-4 flex items-center justify-between gap-3">
            <p className="text-xs text-slate-400">文字数制限は設けていません。</p>
            <button
              className="inline-flex items-center justify-center rounded-full bg-slate-900 px-5 py-3 text-sm font-semibold text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={isPosting || draft.trim().length === 0}
              onClick={onSubmit}
              type="button"
            >
              {isPosting ? "投稿中..." : "投稿する"}
            </button>
          </div>
        </div>
        <div className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-xl">
          <h2 className="text-lg font-semibold text-slate-900">ヒント</h2>
          <ul className="mt-3 space-y-2 text-sm leading-6 text-slate-600">
            <li>・更新ボタンで最新のタイムラインを再取得できます。</li>
            <li>・投稿は認証済みセッションのIDトークン付きで送信されます。</li>
            <li>・長文も投稿できますが、読みやすいよう改行を活用してください。</li>
          </ul>
        </div>
      </section>
    </div>
  );
}

/** タイムライン状態管理と投稿処理を束ねる Container Component。 */
export function ChatPage({ isAuthenticated, isAuthChecking, onNavigate, onToast }: ChatPageProps) {
  const [messages, setMessages] = useState<TimelineMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isPosting, setIsPosting] = useState(false);
  const [hasMore, setHasMore] = useState(true);

  const sortedMessages = useMemo(
    () => [...messages].sort((left, right) => new Date(right.created_at).getTime() - new Date(left.created_at).getTime()),
    [messages],
  );

  /**
   * タイムラインを取得し、必要に応じて既存一覧へ追記する。
   */
  const loadTimeline = useCallback(
    async (before?: string, append = false) => {
      setIsLoading(true);
      try {
        const timeline = await fetchTimeline(before);
        setMessages((current) => {
          if (!append) {
            return timeline;
          }

          const next = [...current, ...timeline];
          const seen = new Set<string>();
          return next.filter((message) => {
            const key = `${message.created_at}-${message.user_name}-${message.body}`;
            if (seen.has(key)) {
              return false;
            }
            seen.add(key);
            return true;
          });
        });
        setHasMore(timeline.length >= 50);
      } catch (error) {
        const message = error instanceof Error ? error.message : "タイムラインの取得に失敗しました。";
        onToast(message, "error");
      } finally {
        setIsLoading(false);
      }
    },
    [onToast],
  );

  useEffect(() => {
    if (!isAuthenticated || isAuthChecking) {
      return;
    }

    queueMicrotask(() => {
      void loadTimeline();
    });
  }, [isAuthenticated, isAuthChecking, loadTimeline]);

  /** 入力中のメッセージを投稿し、成功後に一覧を再取得する。 */
  const handleSubmit = async () => {
    const body = draft.trim();
    if (!body) {
      onToast("投稿内容を入力してください。", "error");
      return;
    }

    setIsPosting(true);
    try {
      await postMessage({ body });
      setDraft("");
      onToast("投稿しました。", "success");
      await loadTimeline();
    } catch (error) {
      const message = error instanceof Error ? error.message : "投稿に失敗しました。";
      onToast(message, "error");
    } finally {
      setIsPosting(false);
    }
  };

  /** 末尾メッセージを境界にして追加読み込みを行う。 */
  const handleLoadMore = async () => {
    const lastMessage = sortedMessages[sortedMessages.length - 1];
    if (!lastMessage) {
      return;
    }

    await loadTimeline(lastMessage.created_at, true);
  };

  return (
    <ChatPageView
      draft={draft}
      hasMore={hasMore}
      isAuthenticated={isAuthenticated}
      isAuthChecking={isAuthChecking}
      isLoading={isLoading}
      isPosting={isPosting}
      messages={sortedMessages}
      onChangeDraft={setDraft}
      onLoadMore={() => void handleLoadMore()}
      onNavigate={onNavigate}
      onRefresh={() => void loadTimeline()}
      onSubmit={() => void handleSubmit()}
    />
  );
}
