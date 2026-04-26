import {
  SorobanRevertError,
  StellarGrantsError,
  getErrorClass,
  ERROR_CLASSES,
} from "./StellarGrantsError";

/**
 * Regular expression to match error codes in Soroban error messages.
 * Matches patterns like: "Error(CodeN)" or "error code: N" or "code: N"
 */
const ERROR_CODE_REGEX = /(?:error|code|Error|Code)[\s:]*\(?(\d+)\)?/i;

/**
 * Converts generic Soroban failures into readable typed errors.
 */
export function parseSorobanError(error: unknown): Error {
  if (error instanceof Error) {
    const msg = error.message;
    const lower = msg.toLowerCase();

    // Check for revert/txFailed errors
    if (lower.includes("revert") || lower.includes("txfailed")) {
      // Try to extract error code from the message
      const errorCode = extractErrorCode(msg);

      if (errorCode !== null && errorCode in ERROR_CLASSES) {
        const ErrorClass = getErrorClass(errorCode);
        if (ErrorClass) {
          return new ErrorClass({
            raw: msg,
            code: errorCode,
          }) as Error;
        }
      }

      // Fall back to generic revert error
      return new SorobanRevertError(humanizeRevertMessage(msg), { raw: msg });
    }

    return new StellarGrantsError(msg, "RPC_ERROR");
  }

  return new StellarGrantsError("Unknown Soroban RPC failure", "UNKNOWN_RPC_ERROR", error);
}

/**
 * Extracts error code from a Soroban error message.
 * Looks for patterns like "Error(Code2)" or "error code: 2" in the message.
 */
function extractErrorCode(message: string): number | null {
  const match = message.match(ERROR_CODE_REGEX);
  if (match && match[1]) {
    const code = parseInt(match[1], 10);
    // Validate the code is within our known range
    if (code >= 1 && code <= 48) {
      return code;
    }
  }
  return null;
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
