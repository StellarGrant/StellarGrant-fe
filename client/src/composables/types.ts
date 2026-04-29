import { StellarGrantsSDK } from "../StellarGrantsSDK";

/**
 * Context value provided by StellarGrantsProvider.
 * Mirrors the React context structure for Vue compatibility.
 */
export interface StellarGrantsContext {
  /** Pre-configured StellarGrantsSDK instance */
  sdk: StellarGrantsSDK;
  /** Optional logger instance for debugging */
  logger?: {
    debug: (...args: unknown[]) => void;
    info: (...args: unknown[]) => void;
    warn: (...args: unknown[]) => void;
    error: (...args: unknown[]) => void;
  };
}

/**
 * Injection key symbol for StellarGrants context.
 * Used internally by provide/inject mechanism.
 */
export const STELLAR_GRANTS_KEY = Symbol("stellar-grants") as InjectionKey<StellarGrantsContext>;

/**
 * Vue injection key type helper.
 * This is a placeholder - in a real Vue project, this would be from 'vue'.
 * For SDK distribution, we use a simpler approach that works with both Vue 2 and Vue 3.
 */
export type InjectionKey<T> = string | symbol;
