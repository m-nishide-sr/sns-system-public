/**
 * Next.js アプリケーション全体の共通レイアウトとメタデータを定義する。
 */
import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "SNSシステム",
  description: "SNSシステムのフロントエンド",
};

/**
 * すべてのページに共通する HTML / body 要素を提供する。
 */
export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="ja">
      <body className="min-h-screen bg-slate-50 text-slate-900 antialiased">
        {children}
      </body>
    </html>
  );
}
