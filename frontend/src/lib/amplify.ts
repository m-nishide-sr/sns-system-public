/**
 * Cognito 用 Amplify 初期化を 1 度だけ実行する。
 */
import { Amplify } from "aws-amplify";

/** 二重初期化を避けるためのフラグ。 */
let configured = false;

/**
 * 環境変数に応じて Amplify Auth を安全に初期化する。
 */
export function ensureAmplifyConfigured() {
  if (configured) {
    return;
  }

  const userPoolId = process.env.NEXT_PUBLIC_COGNITO_USER_POOL_ID;
  const userPoolClientId = process.env.NEXT_PUBLIC_COGNITO_CLIENT_ID;

  if (!userPoolId || !userPoolClientId) {
    console.warn("Cognito設定が不足しているため、認証機能は利用できません。");
    configured = true;
    return;
  }

  Amplify.configure({
    Auth: {
      Cognito: {
        userPoolId,
        userPoolClientId,
      },
    },
  });

  configured = true;
}
