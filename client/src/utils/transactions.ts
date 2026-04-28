import { StrKey, TransactionBuilder, xdr } from "@stellar/stellar-sdk";

function bytesToHex(bytes: Uint8Array): string {
  let out = "";
  for (const b of bytes) {
    out += b.toString(16).padStart(2, "0");
  }
  return out;
}

export function xdrToBase64(value: { toXDR: (format?: "base64") => string } | string): string {
  if (typeof value === "string") return value;
  return value.toXDR("base64");
}

export function xdrFromBase64<T>(base64: string, type: { fromXDR: (xdr: string, format: "base64") => T }): T {
  return type.fromXDR(base64, "base64");
}

export function appendSignature(txXdr: string, signatureXdr: string, networkPassphrase: string): string {
  const tx: any = TransactionBuilder.fromXDR(txXdr, networkPassphrase);
  const decorated = xdr.DecoratedSignature.fromXDR(signatureXdr, "base64");
  if (!Array.isArray((tx as any).signatures)) {
    (tx as any).signatures = [];
  }
  (tx as any).signatures.push(decorated);
  return tx.toXDR();
}

export type AccountThresholds = { low_threshold: number; med_threshold: number; high_threshold: number };
export type AccountSigner = { key: string; weight: number };

export function computeSignatureWeight(
  txXdr: string,
  networkPassphrase: string,
  signers: AccountSigner[],
): number {
  const tx: any = TransactionBuilder.fromXDR(txXdr, networkPassphrase);
  const signatures: xdr.DecoratedSignature[] = (tx as any).signatures ?? [];
  const signatureHints = new Set(signatures.map((s) => bytesToHex(s.hint())));

  let total = 0;
  for (const signer of signers) {
    if (!signer?.key) continue;
    try {
      const raw = StrKey.decodeEd25519PublicKey(signer.key);
      const hint = bytesToHex(raw.slice(raw.length - 4));
      if (signatureHints.has(hint)) {
        total += Number(signer.weight ?? 0);
      }
    } catch {
      // ignore malformed keys
    }
  }
  return total;
}

export function meetsThreshold(weight: number, thresholds: AccountThresholds, level: "low" | "medium" | "high" = "medium"): boolean {
  const required = level === "low"
    ? thresholds.low_threshold
    : level === "high"
      ? thresholds.high_threshold
      : thresholds.med_threshold;
  return weight >= required;
}

export class PendingXdrStore {
  private readonly store = new Map<string, { xdr: string; createdAtMs: number }>();

  save(id: string, xdr: string): void {
    this.store.set(id, { xdr, createdAtMs: Date.now() });
  }

  get(id: string): string | null {
    return this.store.get(id)?.xdr ?? null;
  }

  delete(id: string): void {
    this.store.delete(id);
  }

  clear(): void {
    this.store.clear();
  }
}
