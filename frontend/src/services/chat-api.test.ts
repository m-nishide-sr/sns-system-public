import { fetchChats, postChat } from "@/src/services/chat-api";
import { fetchAccessToken } from "@/src/services/auth";

jest.mock("@/src/services/auth", () => ({
  fetchAccessToken: jest.fn(),
}));

const mockedFetchAccessToken = jest.mocked(fetchAccessToken);

describe("chat-api", () => {
  beforeEach(() => {
    mockedFetchAccessToken.mockReset();
    global.fetch = jest.fn();
  });

  it("チャット取得時にBearerトークン付きでGETする", async () => {
    mockedFetchAccessToken.mockResolvedValue("token-123");
    const fetchMock = jest.mocked(global.fetch);
    fetchMock.mockResolvedValue({
      ok: true,
      json: async () => [{ id: "1", body: "hello", createdAt: "2026-01-01T00:00:00Z" }],
    } as Response);

    const result = await fetchChats();

    expect(fetchMock).toHaveBeenCalledWith("/api/chat", {
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + "token-123",
      },
    });
    expect(result).toHaveLength(1);
  });

  it("チャット投稿時に本文をJSON化してPOSTする", async () => {
    mockedFetchAccessToken.mockResolvedValue("token-456");
    const fetchMock = jest.mocked(global.fetch);
    fetchMock.mockResolvedValue({
      ok: true,
      json: async () => ({ id: "2", body: "posted", createdAt: "2026-01-01T00:00:00Z" }),
    } as Response);

    await postChat("posted");

    expect(fetchMock).toHaveBeenCalledWith("/api/chat", {
      method: "POST",
      body: JSON.stringify({ body: "posted" }),
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + "token-456",
      },
    });
  });
});
