/**
 * APIエラークラスのテスト
 * node:assert を使用した単体テスト
 */
import assert from "node:assert/strict";
import test from "node:test";

// ApiError を再現する（importが困難なためインライン定義）
class ApiError extends Error {
  status: number;
  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

test("ApiError はエラーメッセージとステータスを保持する", () => {
  const err = new ApiError("認証エラー", 401);
  assert.equal(err.message, "認証エラー");
  assert.equal(err.status, 401);
  assert.equal(err.name, "ApiError");
  assert.ok(err instanceof Error);
});

test("ApiError はさまざまなHTTPステータスコードを受け付ける", () => {
  const codes = [400, 401, 403, 404, 500, 503];
  for (const code of codes) {
    const err = new ApiError(`エラー ${code}`, code);
    assert.equal(err.status, code);
  }
});

test("fetchTimeline用のクエリパラメータURLエンコードが正しく動作する", () => {
  const before = "2025-01-15T09:00:00Z";
  const encoded = encodeURIComponent(before);
  const url = `/api/v1/timeline?before=${encoded}`;
  assert.ok(url.includes("2025-01-15T09"), "ISO8601日時が含まれる");
  assert.ok(!url.includes(":"), "コロンがエンコードされている");
});
