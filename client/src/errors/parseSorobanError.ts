import { ContractErrorCode, ErrorMessages } from "./errorCodes";
import {
  SorobanRevertError,
  StellarGrantsError,
  UnauthorizedError,
  GrantNotFoundError,
  InvalidStateError,
} from "./StellarGrantsError";

/**
 * Converts generic Soroban failures into readable typed errors.
 */
export function parseSorobanError(error: unknown): Error {
  if (error instanceof Error) {
    const msg = error.message;
    const lower = msg.toLowerCase();

    if (lower.includes("revert") || lower.includes("txfailed")) {
      const code = extractErrorCode(msg);
      if (code !== null) {
        return mapContractError(code, msg);
      }
      return new SorobanRevertError(humanizeRevertMessage(msg), { raw: msg });
    }
    return new StellarGrantsError(msg, "RPC_ERROR");
  }

  return new StellarGrantsError(
    "Unknown Soroban RPC failure",
    "UNKNOWN_RPC_ERROR",
    error,
  );
}

function extractErrorCode(msg: string): number | null {
  // Matches "Error(Contract, #1)" or "Error(Contract, 1)" or "revert: 1"
  const match = msg.match(/Error\(Contract, #?(\d+)\)/) || msg.match(/revert:\s*(\d+)/i);
  return match ? parseInt(match[1], 10) : null;
}

function mapContractError(code: number, rawMsg: string): Error {
  const message =
    ErrorMessages[code as ContractErrorCode] || `Contract error code: ${code}`;
  const details = { code, raw: rawMsg };

  switch (code) {
    case ContractErrorCode.Unauthorized:
      return new UnauthorizedError(details);
    case ContractErrorCode.GrantNotFound:
      return new GrantNotFoundError(details);
    case ContractErrorCode.InvalidState:
      return new InvalidStateError(message, details);
    default:
      return new SorobanRevertError(message, details);
  }
}

function humanizeRevertMessage(raw: string): string {
  const reasonMatch = raw.match(/revert(?:ed)?[:\s-]+(.+)/i);
  if (reasonMatch?.[1]) {
    return `Contract reverted: ${reasonMatch[1]}`;
  }

  const txFailedMatch = raw.match(/txfailed[:\s-]+(.+)/i);
  if (txFailedMatch?.[1]) {
    return `Transaction failed: ${txFailedMatch[1]}`;
  }

  return "Contract reverted while executing request";
}
