import {
  confirmSignUp,
  deleteUser,
  fetchAuthSession,
  getCurrentUser,
  signIn,
  signOut,
  signUp,
  updatePassword,
} from "aws-amplify/auth";
import type { AuthUser } from "@/src/types/app";
import { configureAmplify } from "@/src/services/amplify";

function getErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  return "不明なエラーが発生しました。";
}

export async function fetchSignedInUser(): Promise<AuthUser | null> {
  configureAmplify();

  try {
    const currentUser = await getCurrentUser();
    const session = await fetchAuthSession();
    const idToken = session.tokens?.idToken?.payload;

    return {
      displayName:
        typeof idToken?.email === "string"
          ? idToken.email
          : currentUser.username,
      email: typeof idToken?.email === "string" ? idToken.email : undefined,
    };
  } catch {
    return null;
  }
}

export async function signInWithEmail(email: string, password: string) {
  configureAmplify();
  const result = await signIn({
    username: email,
    password,
  });

  return result;
}

export async function signUpWithEmail(email: string, password: string) {
  configureAmplify();

  return signUp({
    username: email,
    password,
    options: {
      userAttributes: {
        email,
      },
    },
  });
}

export async function confirmSignUpCode(email: string, confirmationCode: string) {
  configureAmplify();

  return confirmSignUp({
    username: email,
    confirmationCode,
  });
}

export async function signOutCurrentUser() {
  configureAmplify();
  await signOut();
}

export async function changePassword(oldPassword: string, newPassword: string) {
  configureAmplify();
  await updatePassword({
    oldPassword,
    newPassword,
  });
}

export async function deleteCurrentUser() {
  configureAmplify();
  await deleteUser();
}

export async function fetchAccessToken() {
  configureAmplify();
  const session = await fetchAuthSession();

  return session.tokens?.accessToken?.toString() ?? null;
}

export function normalizeAuthError(error: unknown) {
  return getErrorMessage(error);
}
