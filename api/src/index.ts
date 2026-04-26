import { createServer } from "http";
import { env } from "./config/env";
import { createApp } from "./app";
import { buildDataSource } from "./db/data-source";
import { MockSorobanContractClient } from "./soroban/mock-client";
import { notificationService } from "./services/notification-service";
import { ReconciliationService } from "./services/reconciliation-service";
import { GrantSyncService } from "./services/grant-sync-service";
import { logger } from "./config/logger";

const bootstrap = async () => {
  const dataSource = buildDataSource();
  await dataSource.initialize();

  const sorobanClient = new MockSorobanContractClient();
  const app = createApp(dataSource, sorobanClient);
  const server = createServer(app);

  // Initialize real-time notifications
  notificationService.initialize(server);

  // Start the periodic reconciliation task (every 30 minutes)
  const grantSyncService = new GrantSyncService(dataSource, sorobanClient);
  const reconciliationService = new ReconciliationService(dataSource, sorobanClient, grantSyncService);
  reconciliationService.start(30 * 60 * 1000);

  server.listen(env.port, () => {
    logger.info(`API listening on port ${env.port}`);
  });

  // Graceful shutdown
  const shutdown = () => {
    reconciliationService.stop();
    server.close(() => process.exit(0));
  };
  process.once("SIGTERM", shutdown);
  process.once("SIGINT", shutdown);
};

bootstrap().catch((error) => {
  console.error("Failed to start API", error);
  process.exit(1);
});
