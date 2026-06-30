import { Amplify } from "aws-amplify";

let configured = false;

function getEnv(name: string) {
  return process.env[name]?.trim();
}

export function getFrontendConfig() {
  return {
    region: getEnv("NEXT_PUBLIC_AWS_REGION"),
    userPoolId: getEnv("NEXT_PUBLIC_COGNITO_USER_POOL_ID"),
    userPoolClientId: getEnv("NEXT_PUBLIC_COGNITO_USER_POOL_WEB_CLIENT_ID"),
    allowDomain: getEnv("NEXT_PUBLIC_ALLOW_DOMAIN"),
  };
}

export function configureAmplify() {
  if (configured) {
    return;
  }

  const config = getFrontendConfig();

  if (!config.region || !config.userPoolId || !config.userPoolClientId) {
    return;
  }

  Amplify.configure({
    Auth: {
      Cognito: {
        userPoolId: config.userPoolId,
        userPoolClientId: config.userPoolClientId,
        loginWith: {
          email: true,
        },
      },
    },
  });

  configured = true;
}
