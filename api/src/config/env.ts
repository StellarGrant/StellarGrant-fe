import dotenv from "dotenv";

dotenv.config();

export const env = {
  port: Number(process.env.PORT ?? 4000),
  databaseUrl:
    process.env.DATABASE_URL ?? "postgres://postgres:postgres@localhost:5432/stellargrant",
  adminAddresses: (process.env.ADMIN_ADDRESSES ?? "").split(",").map(a => a.trim()).filter(Boolean),
  nodeEnv: process.env.NODE_ENV ?? "development",
  corsOrigins: (process.env.CORS_ORIGINS ?? "http://localhost:3000").split(",").map(a => a.trim()).filter(Boolean),
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
};
