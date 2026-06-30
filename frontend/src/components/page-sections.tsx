import type { ChatMessage } from "@/src/types/app";

type HomeSectionProps = {
  loginHref: string;
  termsHref: string;
};

export function HomeSection({ loginHref, termsHref }: HomeSectionProps) {
  return (
    <section className="hero page-card">
      <div>
        <p className="note">信頼できる利用者だけで会話できるシンプルなSNSです。</p>
        <h1>社内・限定コミュニティ向けの静的SPAチャット</h1>
        <p>
          Cognito認証でサインインし、API経由でチャットを取得・投稿できます。スマートフォンでもPCでも読みやすいようにレスポンシブで設計しています。
        </p>
      </div>
      <div className="hero-actions">
        <a className="primary-button" href={loginHref}>
          ログイン / 新規登録へ進む
        </a>
        <a className="secondary-button" href={termsHref}>
          利用規約を確認する
        </a>
      </div>
      <ul className="hero-highlights">
        <li>ログイン後はマイページからパスワード変更・退会・ログアウトが可能です。</li>
        <li>チャットページでは初期表示時と更新ボタンで最新一覧を取得します。</li>
        <li>投稿時の文字数制限は設けず、相対パス <code>/api/chat</code> を利用します。</li>
      </ul>
    </section>
  );
}

type LoginSectionProps = {
  allowDomain: string;
  signInEmail: string;
  signInPassword: string;
  signUpEmail: string;
  signUpPassword: string;
  confirmationEmail: string;
  confirmationCode: string;
  onSignInEmailChange: (value: string) => void;
  onSignInPasswordChange: (value: string) => void;
  onSignUpEmailChange: (value: string) => void;
  onSignUpPasswordChange: (value: string) => void;
  onConfirmationEmailChange: (value: string) => void;
  onConfirmationCodeChange: (value: string) => void;
  onSignIn: () => void;
  onSignUp: () => void;
  onConfirm: () => void;
};

export function LoginSection({
  allowDomain,
  signInEmail,
  signInPassword,
  signUpEmail,
  signUpPassword,
  confirmationEmail,
  confirmationCode,
  onSignInEmailChange,
  onSignInPasswordChange,
  onSignUpEmailChange,
  onSignUpPasswordChange,
  onConfirmationEmailChange,
  onConfirmationCodeChange,
  onSignIn,
  onSignUp,
  onConfirm,
}: LoginSectionProps) {
  return (
    <div className="split-grid">
      <section className="split-card">
        <h2>ログイン</h2>
        <p className="note">登録済みのメールアドレスとパスワードでサインインしてください。</p>
        <div className="field-grid">
          <label className="field-label">
            <span>メールアドレス</span>
            <input
              className="text-field"
              type="email"
              value={signInEmail}
              onChange={(event) => onSignInEmailChange(event.target.value)}
            />
          </label>
          <label className="field-label">
            <span>パスワード</span>
            <input
              className="text-field"
              type="password"
              value={signInPassword}
              onChange={(event) => onSignInPasswordChange(event.target.value)}
            />
          </label>
          <div className="card-actions">
            <button className="primary-button" type="button" onClick={onSignIn}>
              ログインする
            </button>
          </div>
        </div>
      </section>
      <section className="split-card">
        <h2>新規登録</h2>
        <p className="note">
          登録できるメールアドレスは <strong>{allowDomain}</strong> ドメインのみです。実際のドメイン検証はCognitoのPreSignUpでも実施されます。
        </p>
        <div className="field-grid">
          <label className="field-label">
            <span>メールアドレス</span>
            <input
              className="text-field"
              type="email"
              value={signUpEmail}
              onChange={(event) => onSignUpEmailChange(event.target.value)}
            />
          </label>
          <label className="field-label">
            <span>パスワード</span>
            <input
              className="text-field"
              type="password"
              value={signUpPassword}
              onChange={(event) => onSignUpPasswordChange(event.target.value)}
            />
          </label>
          <div className="card-actions">
            <button className="primary-button" type="button" onClick={onSignUp}>
              新規登録する
            </button>
          </div>
        </div>
      </section>
      <section className="split-card">
        <h2>確認コード入力</h2>
        <p className="note">
          新規登録直後、またはメールアドレス未承認の状態では、受信した確認コードを入力してください。
        </p>
        <div className="field-grid">
          <label className="field-label">
            <span>メールアドレス</span>
            <input
              className="text-field"
              type="email"
              value={confirmationEmail}
              onChange={(event) => onConfirmationEmailChange(event.target.value)}
            />
          </label>
          <label className="field-label">
            <span>確認コード</span>
            <input
              className="text-field"
              type="text"
              value={confirmationCode}
              onChange={(event) => onConfirmationCodeChange(event.target.value)}
            />
          </label>
          <div className="card-actions">
            <button className="primary-button" type="button" onClick={onConfirm}>
              コードを確認する
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}

type ChatSectionProps = {
  messages: ChatMessage[];
  draft: string;
  signedIn: boolean;
  onDraftChange: (value: string) => void;
  onRefresh: () => void;
  onSubmit: () => void;
};

export function ChatSection({
  messages,
  draft,
  signedIn,
  onDraftChange,
  onRefresh,
  onSubmit,
}: ChatSectionProps) {
  return (
    <section className="page-card">
      <div className="stack-row">
        <button className="secondary-button" type="button" onClick={onRefresh}>
          更新する
        </button>
        {!signedIn ? (
          <div className="status-card">チャット機能を利用するにはログインしてください。</div>
        ) : null}
      </div>
      <div className="field-grid">
        <label className="field-label">
          <span>投稿内容</span>
          <textarea
            className="text-area"
            value={draft}
            onChange={(event) => onDraftChange(event.target.value)}
            placeholder="ここにメッセージを入力してください。"
          />
        </label>
        <div className="card-actions">
          <button className="primary-button" type="button" onClick={onSubmit}>
            投稿する
          </button>
        </div>
      </div>
      <div className="chat-list">
        {messages.length === 0 ? (
          <div className="status-card">表示できるチャットがまだありません。</div>
        ) : (
          messages.map((message) => (
            <article className="chat-item" key={message.id}>
              <p>{message.body}</p>
              <time dateTime={message.createdAt}>
                {new Date(message.createdAt).toLocaleString("ja-JP")}
              </time>
            </article>
          ))
        )}
      </div>
    </section>
  );
}

type MyPageSectionProps = {
  userLabel: string;
  currentPassword: string;
  nextPassword: string;
  onCurrentPasswordChange: (value: string) => void;
  onNextPasswordChange: (value: string) => void;
  onChangePassword: () => void;
  onDeleteAccount: () => void;
  onSignOut: () => void;
};

export function MyPageSection({
  userLabel,
  currentPassword,
  nextPassword,
  onCurrentPasswordChange,
  onNextPasswordChange,
  onChangePassword,
  onDeleteAccount,
  onSignOut,
}: MyPageSectionProps) {
  return (
    <div className="split-grid">
      <section className="split-card">
        <h2>アカウント情報</h2>
        <div className="status-card">現在のログイン名: {userLabel}</div>
      </section>
      <section className="split-card">
        <h2>パスワード変更</h2>
        <div className="field-grid">
          <label className="field-label">
            <span>現在のパスワード</span>
            <input
              className="text-field"
              type="password"
              value={currentPassword}
              onChange={(event) => onCurrentPasswordChange(event.target.value)}
            />
          </label>
          <label className="field-label">
            <span>新しいパスワード</span>
            <input
              className="text-field"
              type="password"
              value={nextPassword}
              onChange={(event) => onNextPasswordChange(event.target.value)}
            />
          </label>
          <div className="card-actions">
            <button className="primary-button" type="button" onClick={onChangePassword}>
              パスワードを変更する
            </button>
          </div>
        </div>
      </section>
      <section className="split-card">
        <h2>セッション管理</h2>
        <p className="note">ログアウト後もアカウント自体は残ります。</p>
        <div className="card-actions">
          <button className="secondary-button" type="button" onClick={onSignOut}>
            ログアウトする
          </button>
        </div>
      </section>
      <section className="split-card">
        <h2>退会</h2>
        <p className="note warning-note">
          退会ではCognitoのユーザー削除のみを行い、チャットデータの削除は実施しません。
        </p>
        <div className="card-actions">
          <button className="danger-button" type="button" onClick={onDeleteAccount}>
            退会する
          </button>
        </div>
      </section>
    </div>
  );
}

export function TermsSection() {
  return (
    <section className="page-card">
      <h2>利用規約（暫定版）</h2>
      <p className="note">
        本ページの文面は暫定版です。正式な利用規約は別途整備予定ですが、現時点では以下の内容に同意のうえご利用ください。
      </p>
      <ul className="bullet-list">
        <li>投稿した内容は一度登録すると修正も削除もできません。</li>
        <li>
          パスワードが流出し、攻撃者によりパスワードが不正に変更された場合、アカウントを回復できない可能性があります。
        </li>
        <li>法令や公序良俗に反する投稿、第三者への迷惑行為は禁止します。</li>
        <li>運営は必要に応じてサービス内容を変更または停止できるものとします。</li>
      </ul>
    </section>
  );
}
