import { buildHash, resolveRoute, routeDefinitions } from "@/src/lib/routes";

describe("routes", () => {
  it("ハッシュからページを判定できる", () => {
    expect(resolveRoute("")).toBe("home");
    expect(resolveRoute("#/login")).toBe("login");
    expect(resolveRoute("#/chat")).toBe("chat");
    expect(resolveRoute("#/unknown")).toBe("home");
  });

  it("ヘッダー表示有無とタイトルを持つ", () => {
    expect(routeDefinitions.home.showHeader).toBe(false);
    expect(routeDefinitions.chat.title).toBe("チャット");
  });

  it("ルートからハッシュを組み立てる", () => {
    expect(buildHash("home")).toBe("#/");
    expect(buildHash("terms")).toBe("#/terms");
  });
});
