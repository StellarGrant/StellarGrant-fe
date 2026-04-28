import dotenv from "dotenv";

dotenv.config();

export const env = {
  port: Number(process.env.PORT ?? 4000),
  databaseUrl:
    process.env.DATABASE_URL ?? "postgres://postgres:postgres@localhost:5432/stellargrant",
  adminAddresses: (process.env.ADMIN_ADDRESSES ?? "").split(",").map((a: string) => a.trim()).filter(Boolean),
  nodeEnv: process.env.NODE_ENV ?? "development",
  corsOrigins: (process.env.CORS_ORIGINS ?? "http://localhost:3000").split(",").map((a: string) => a.trim()).filter(Boolean),
  logLevel: process.env.LOG_LEVEL ?? (process.env.NODE_ENV === "production" ? "info" : "debug"),
  // IPFS / Pinata
  pinataJwt: process.env.PINATA_JWT ?? "",
  ipfsGateway: process.env.IPFS_GATEWAY ?? "https://gateway.pinata.cloud",
  // Email / SMTP
  smtpHost: process.env.SMTP_HOST ?? "localhost",
  smtpPort: Number(process.env.SMTP_PORT ?? 587),
  smtpUser: process.env.SMTP_USER ?? "",
  smtpPass: process.env.SMTP_PASS ?? "",
  emailFrom: process.env.EMAIL_FROM ?? "noreply@stellargrant.io",
  appBaseUrl: process.env.APP_BASE_URL ?? "https://stellargrant.io",
  /** When empty, response caching is disabled (safe for tests and local dev without Redis). */
  redisUrl: process.env.REDIS_URL ?? "",
  cacheTtlSeconds: Math.max(5, Number(process.env.CACHE_TTL_SECONDS ?? 120)),
  rateLimitAlertWindowMinutes: Math.max(1, Number(process.env.RATE_LIMIT_ALERT_WINDOW_MINUTES ?? 5)),
  rateLimitAlertThreshold: Math.max(1, Number(process.env.RATE_LIMIT_ALERT_THRESHOLD ?? 200)),
  rateLimitAlertCooldownMinutes: Math.max(1, Number(process.env.RATE_LIMIT_ALERT_COOLDOWN_MINUTES ?? 10)),
  reconciliationIntervalMinutes: Math.max(1, Number(process.env.RECONCILIATION_INTERVAL_MINUTES ?? 30)),
  metricsBasicAuthUser: process.env.METRICS_BASIC_AUTH_USER ?? "",
  metricsBasicAuthPassword: process.env.METRICS_BASIC_AUTH_PASSWORD ?? "",
  metricsAllowedIps: (process.env.METRICS_ALLOWED_IPS ?? "127.0.0.1,::1,::ffff:127.0.0.1")
    .split(",")
    .map((ip: string) => ip.trim())
    .filter(Boolean),
  /** Optional JWT secret used to validate WebSocket authentication tokens. */
  jwtSecret: process.env.JWT_SECRET ?? "",
};
