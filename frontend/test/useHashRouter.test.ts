/**
 * ハッシュルーターの正規化規則と許可ルート一覧を検証するテスト。
 */
import assert from "node:assert/strict";
import test from "node:test";
import { normalizeHash, ROUTES } from "../src/hooks/useHashRouter.ts";

test("ROUTES に必要な画面が含まれる", () => {
  assert.deepEqual(ROUTES, ["/", "/chat", "/login", "/mypage", "/terms"]);
});

test("normalizeHash がハッシュを画面ルートへ正規化する", () => {
  assert.equal(normalizeHash(""), "/");
  assert.equal(normalizeHash("#/chat"), "/chat");
  assert.equal(normalizeHash("terms"), "/terms");
  assert.equal(normalizeHash("#/unknown"), "/");
});
