import { Amplify } from "aws-amplify";

let configured = false;

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
