import { useCallback, useEffect, useMemo, useState } from "react";

export type ToastTone = "success" | "error";

export type ToastItem = {
  id: string;
  tone: ToastTone;
  message: string;
};

export function useToast() {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const removeToast = useCallback((id: string) => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  const showToast = useCallback((message: string, tone: ToastTone = "success") => {
    const id = `${Date.now()}-${Math.random().toString(16).slice(2)}`;
    setToasts((current) => [...current, { id, message, tone }]);
    return id;
  }, []);

  return useMemo(() => ({ toasts, showToast, removeToast }), [removeToast, showToast, toasts]);
}

type ToastCardProps = {
  toast: ToastItem;
  onClose: (id: string) => void;
};

export function ToastCard({ toast, onClose }: ToastCardProps) {
  useEffect(() => {
    const timer = window.setTimeout(() => onClose(toast.id), 3000);
    return () => window.clearTimeout(timer);
  }, [onClose, toast.id]);

  return (
    <div
      className={`pointer-events-auto flex items-start gap-3 rounded-2xl border px-4 py-3 shadow-xl backdrop-blur ${toast.tone === "success" ? "border-emerald-200 bg-emerald-50 text-emerald-900" : "border-rose-200 bg-rose-50 text-rose-900"}`}
      role="status"
    >
      <span className="pt-0.5 text-lg">{toast.tone === "success" ? "✓" : "!"}</span>
      <div className="min-w-0 flex-1 text-sm font-medium">{toast.message}</div>
      <button
        aria-label="通知を閉じる"
        className="text-xs text-slate-500 transition hover:text-slate-900"
        onClick={() => onClose(toast.id)}
        type="button"
      >
        閉じる
      </button>
    </div>
  );
}

type ToastViewportProps = {
  toasts: ToastItem[];
  onClose: (id: string) => void;
};

export function ToastViewport({ toasts, onClose }: ToastViewportProps) {
  return (
    <div className="pointer-events-none fixed right-4 top-4 z-[60] flex w-[min(24rem,calc(100vw-2rem))] flex-col gap-3">
      {toasts.map((toast) => (
        <ToastCard key={toast.id} toast={toast} onClose={onClose} />
      ))}
    </div>
  );
}
