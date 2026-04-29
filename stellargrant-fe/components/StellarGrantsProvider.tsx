"use client";

/**
 * StellarGrantsProvider
 *
 * Top-level React context that manages the SDK lifecycle: contract client,
 * wallet connection state, configurable logger, and a shared BatchBuilder.
 * Wrap your app (or a subtree) with this provider and consume the context
 * via `useStellarGrants()`.
 */

import { createContext, useMemo, type ReactNode } from "react";
import { ContractClient, contractClient as defaultClient } from "@/lib/stellar/contract";
import { Logger, type LogLevel } from "@/lib/logger";
import { BatchBuilder } from "@/lib/stellar/batchBuilder";

export interface StellarGrantsContextValue {
  /** Pre-configured ContractClient instance */
  client: ContractClient;
  /** SDK logger (child of the global logger) */
  logger: Logger;
  /** Shared BatchBuilder for the current render subtree */
  batch: BatchBuilder;
}

export const StellarGrantsContext = createContext<StellarGrantsContextValue | null>(null);

export interface StellarGrantsProviderProps {
  children: ReactNode;
  /** Override the default contract client (useful in tests) */
  client?: ContractClient;
  /** Minimum log level for SDK output. Defaults to "warn" (silent in prod) */
  logLevel?: LogLevel;
  /** Enable verbose debug logging — equivalent to logLevel="debug" */
  debug?: boolean;
}

export function StellarGrantsProvider({
  children,
  client,
  logLevel,
  debug = false,
}: StellarGrantsProviderProps) {
  const ctx = useMemo<StellarGrantsContextValue>(() => {
    const level: LogLevel = debug ? "debug" : (logLevel ?? "warn");
    const sdkLogger = new Logger({ level, prefix: "[StellarGrants]" });
    sdkLogger.info("StellarGrantsProvider mounted", { level });

    return {
      client: client ?? defaultClient,
      logger: sdkLogger,
      batch: new BatchBuilder(),
    };
  }, [client, logLevel, debug]);

  return (
    <StellarGrantsContext.Provider value={ctx}>
      {children}
    </StellarGrantsContext.Provider>
  );
}
