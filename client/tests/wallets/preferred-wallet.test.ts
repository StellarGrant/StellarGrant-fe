import { AlbedoAdapter } from "../../src/wallets/AlbedoAdapter";
import { FreighterAdapter } from "../../src/wallets/FreighterAdapter";
import { createPreferredWalletAdapter } from "../../src/wallets/createPreferredWalletAdapter";

afterEach(() => {
  delete (global as any).window;
});

describe("createPreferredWalletAdapter", () => {
  it("returns FreighterAdapter when Freighter is available", () => {
    (global as any).window = { freighterApi: {} };

    const adapter = createPreferredWalletAdapter();

    expect(adapter).toBeInstanceOf(FreighterAdapter);
  });

  it("falls back to AlbedoAdapter when Freighter is unavailable", () => {
    (global as any).window = { albedo: {} };

    const adapter = createPreferredWalletAdapter();

    expect(adapter).toBeInstanceOf(AlbedoAdapter);
  });

  it("throws when no supported wallet exists", () => {
    (global as any).window = {};

    expect(() => createPreferredWalletAdapter()).toThrow(
      "No supported wallet detected",
    );
  });
});
