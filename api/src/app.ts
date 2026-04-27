import { buildMyDonationsRouter } from "./routes/my-donations";
import express from "express";
import helmet from "helmet";
import cors from "cors";
import morgan from "morgan";
import { DataSource } from "typeorm";
import { Grant } from "./entities/Grant";
import { MilestoneProof } from "./entities/MilestoneProof";
import { User } from "./entities/User";
import { buildUserRouter } from "./routes/users";
import { GrantReviewer } from "./entities/GrantReviewer";
import { buildGrantReviewerRouter } from "./routes/grant-reviewers";
import { MilestoneApproval } from "./entities/MilestoneApproval";
import { buildMilestoneApprovalRouter } from "./routes/milestone-approvals";
 import { buildMilestoneApprovalNotifyRouter } from "./routes/milestone-approvals-notify";
import { Activity } from "./entities/Activity";
import { buildGrantRouter } from "./routes/grants";
import { buildMilestoneProofRouter } from "./routes/milestone-proof";
import { buildLeaderboardRouter } from "./routes/leaderboard";
import { buildAdminRouter } from "./routes/admin";
import { buildActivityRouter } from "./routes/activity";
import { buildProofsRouter } from "./routes/proofs";
import { buildNotificationsRouter } from "./routes/notifications";
import { buildAnalyticsRouter } from "./routes/analytics";
import { buildStatsRouter } from "./routes/stats";
import { GrantSyncService } from "./services/grant-sync-service";
import { ResponseCacheService } from "./services/response-cache";
import { buildSearchRouter } from "./routes/search";
import { buildWatchlistRouter } from "./routes/watchlist";
import { buildDashboardRouter } from "./routes/dashboard";
import { UserWatchlist } from "./entities/UserWatchlist";
import { buildProfilesRouter } from "./routes/profiles";
import { ReconciliationService } from "./services/reconciliation-service";
import { LeaderboardService } from "./services/leaderboard-service";
import { SignatureService } from "./services/signature-service";
import { IpfsService } from "./services/ipfs-service";
import { Contributor } from "./entities/Contributor";
import { AuditLog } from "./entities/AuditLog";
import { GrantView } from "./entities/GrantView";
import { PlatformConfig } from "./entities/PlatformConfig";
import { FeeCollection } from "./entities/FeeCollection";
import { Milestone } from "./entities/Milestone";
import { ConfigService } from "./services/config-service";
import { FeeService } from "./services/fee-service";
import { buildAdminMiddleware } from "./middlewares/admin-middleware";
import { SorobanContractClient } from "./soroban/types";
import { createRateLimiter } from "./middlewares/rate-limiter";
import { errorHandler, notFoundHandler } from "./middlewares/error-handler";
import { env } from "./config/env";
import { requestLogger } from "./config/logger";
import { v4 as uuidv4 } from "uuid";

export const createApp = (dataSource: DataSource, sorobanClient: SorobanContractClient) => {
  const app = express();

  // Security headers with Helmet
  app.use(helmet({
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        scriptSrc: ["'self'"],
        styleSrc: ["'self'", "'unsafe-inline'"],
        imgSrc: ["'self'", "data:", "https:"],
        connectSrc: ["'self'"],
        fontSrc: ["'self'"],
        objectSrc: ["'none'"],
        mediaSrc: ["'self'"],
        frameSrc: ["'none'"],
      },
    },
    crossOriginEmbedderPolicy: false,
  }));

  // CORS configuration
  app.use(cors({
    origin: env.corsOrigins,
    credentials: true,
    methods: ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"],
    allowedHeaders: ["Content-Type", "Authorization", "x-user-address", "x-admin-address", "x-admin-signature", "x-admin-nonce", "x-admin-timestamp"],
  }));

  // Request ID generation
  app.use((req, _res, next) => {
    req.headers["x-request-id"] = req.headers["x-request-id"] || uuidv4();
    next();
  });

  // HTTP request logging with Morgan and Winston
  const httpLogger = requestLogger();
  app.use(morgan("combined", {
    stream: {
      write: (message: string) => {
        httpLogger.info(message.trim());
      },
    },
  }));

  app.use(express.json());

  const rateLimiter = createRateLimiter(dataSource);

  const activityRepo = dataSource.getRepository(Activity);
  const grantRepo = dataSource.getRepository(Grant);
  const milestoneRepo = dataSource.getRepository(Milestone);
  const proofRepo = dataSource.getRepository(MilestoneProof);
  const userRepo = dataSource.getRepository(User);
  const grantReviewerRepo = dataSource.getRepository(GrantReviewer);
  const milestoneApprovalRepo = dataSource.getRepository(MilestoneApproval);
  const grantSyncService = new GrantSyncService(dataSource, sorobanClient);
  const reconciliationService = new ReconciliationService(dataSource, sorobanClient, grantSyncService);
  const signatureService = new SignatureService();
  const leaderboardService = new LeaderboardService(dataSource);

  const contributorRepo = dataSource.getRepository(Contributor);
  const auditLogRepo = dataSource.getRepository(AuditLog);
  const grantViewRepo = dataSource.getRepository(GrantView);
  const ipfsService = new IpfsService();
  const configRepo = dataSource.getRepository(PlatformConfig);
  const feeRepo = dataSource.getRepository(FeeCollection);
  const configService = new ConfigService(configRepo);
  const feeService = new FeeService(feeRepo, configRepo);
  const adminMiddleware = buildAdminMiddleware(signatureService);

  // Health check endpoint (no versioning)
  app.get("/health", (_req, res) => res.json({ ok: true, version: "v1" }));

  // Apply rate limiting
  app.use(rateLimiter);
  app.use("/grants", buildGrantRouter(grantRepo, grantSyncService, signatureService));
  app.use("/milestone_proof", buildMilestoneProofRouter(proofRepo, signatureService, grantRepo, userRepo));
  app.use("/users", buildUserRouter(userRepo));
  app.use("/grant_reviewers", buildGrantReviewerRouter(grantReviewerRepo));
  app.use("/milestone_approvals", buildMilestoneApprovalRouter(milestoneApprovalRepo));
   app.use("/milestone_approvals_notify", buildMilestoneApprovalNotifyRouter(milestoneApprovalRepo, grantRepo, userRepo));
  app.use("/grants", buildGrantRouter(grantRepo, grantSyncService, signatureService));
  app.use("/milestone_proof", buildMilestoneProofRouter(proofRepo, signatureService));
  app.use("/leaderboard", buildLeaderboardRouter(leaderboardService));
  app.use("/activity", buildActivityRouter(activityRepo, contributorRepo));
  app.use(
    "/admin",
    adminMiddleware,
    buildAdminRouter(grantSyncService, contributorRepo, auditLogRepo, responseCache, reconciliationService),
  );
  app.use("/proofs", buildProofsRouter(ipfsService));
  app.use("/notifications", buildNotificationsRouter(contributorRepo));
  app.use("/dashboard", buildDashboardRouter(grantRepo, milestoneRepo, proofRepo, grantSyncService));
  app.use("/analytics", buildAnalyticsRouter(grantRepo, grantViewRepo));
  app.use("/search", buildSearchRouter(dataSource));
  app.use("/profiles", buildProfilesRouter(contributorRepo));
  app.use("/watchlist", buildWatchlistRouter(dataSource.getRepository(UserWatchlist), grantRepo));
  app.use(buildMyDonationsRouter(dataSource));
  app.get("/config/fee", async (req, res) => {
    const fee = await configService.getFeePercentage();
    res.json({ feePercentage: fee });
  });

  app.use((err: unknown, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
    const message = err instanceof Error ? err.message : "Internal server error";
    res.status(500).json({ error: message });
  });

  return app;
};
