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
import { Community } from "./entities/Community";
import { MilestoneComment } from "./entities/MilestoneComment";
import { ConfigService } from "./services/config-service";
import { FeeService } from "./services/fee-service";
import { buildAdminMiddleware } from "./middlewares/admin-middleware";
import { SorobanContractClient } from "./soroban/types";
import { createRateLimiter } from "./middlewares/rate-limiter";
import { errorHandler, notFoundHandler } from "./middlewares/error-handler";
import { metricsMiddleware } from "./middlewares/metrics-middleware";
import { env } from "./config/env";
import { requestLogger } from "./config/logger";
import { v4 as uuidv4 } from "uuid";
import { metricsService } from "./services/metrics-service";
import { buildCommunitiesRouter } from "./routes/communities";
import { buildMilestoneCommentsRouter } from "./routes/milestone-comments";

export const createApp = (dataSource: DataSource, sorobanClient: SorobanContractClient) => {
  const app = express();

  const isMetricsAuthorized = (req: express.Request) => {
    const ip = req.ip || req.socket.remoteAddress || "";
    const ipAllowed = env.metricsAllowedIps.includes(ip);

    if (ipAllowed) return true;

    if (env.metricsBasicAuthUser && env.metricsBasicAuthPassword) {
      const auth = req.header("authorization");
      if (!auth?.startsWith("Basic ")) return false;
      const decoded = Buffer.from(auth.substring(6), "base64").toString("utf8");
      const [user, pass] = decoded.split(":");
      return user === env.metricsBasicAuthUser && pass === env.metricsBasicAuthPassword;
    }

    return false;
  };

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

  app.use(metricsMiddleware);

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
  const responseCache = new ResponseCacheService(env.redisUrl);

  const contributorRepo = dataSource.getRepository(Contributor);
  const auditLogRepo = dataSource.getRepository(AuditLog);
  const grantViewRepo = dataSource.getRepository(GrantView);
  const ipfsService = new IpfsService();
  const configRepo = dataSource.getRepository(PlatformConfig);
  const feeRepo = dataSource.getRepository(FeeCollection);
  const communityRepo = dataSource.getRepository(Community);
  const milestoneCommentRepo = dataSource.getRepository(MilestoneComment);
  const configService = new ConfigService(configRepo);
  const feeService = new FeeService(feeRepo, configRepo);
  const adminMiddleware = buildAdminMiddleware(signatureService);

  // Health check endpoint (no versioning)
  app.get("/health", async (_req, res) => {
    const health = {
      ok: true,
      version: "v1",
      services: {
        database: "ok" as "ok" | "error",
        soroban: "ok" as "ok" | "error",
      },
    };

    try {
      // Check database connectivity
      await dataSource.query("SELECT 1");
    } catch (error) {
      health.services.database = "error";
      health.ok = false;
    }

    try {
      // Check Soroban RPC status
      await sorobanClient.getLatestLedger();
    } catch (error) {
      health.services.soroban = "error";
      health.ok = false;
    }

    const statusCode = health.ok ? 200 : 503;
    res.status(statusCode).json(health);
  });
  app.get("/metrics", async (req, res) => {
    if (!isMetricsAuthorized(req)) {
      res.setHeader("WWW-Authenticate", "Basic realm=metrics");
      res.status(401).json({ error: "Unauthorized" });
      return;
    }

    res.setHeader("Content-Type", metricsService.getContentType());
    res.send(await metricsService.getMetricsText());
  });

  // Apply rate limiting
  app.use(rateLimiter);
  app.use("/grants", buildGrantRouter(grantRepo, milestoneRepo, proofRepo, grantSyncService, signatureService, responseCache));
  app.use("/milestone_proof", buildMilestoneProofRouter(proofRepo, signatureService, responseCache, grantRepo, userRepo));
  app.use("/users", buildUserRouter(userRepo));
  app.use("/grant_reviewers", buildGrantReviewerRouter(grantReviewerRepo));
  app.use("/milestone_approvals", buildMilestoneApprovalRouter(milestoneApprovalRepo));
   app.use("/milestone_approvals_notify", buildMilestoneApprovalNotifyRouter(milestoneApprovalRepo, grantRepo, userRepo));
  app.use("/leaderboard", buildLeaderboardRouter(leaderboardService));
  app.use("/activity", buildActivityRouter(activityRepo, contributorRepo));
  app.use(
    "/admin",
    adminMiddleware,
    buildAdminRouter(grantSyncService, contributorRepo, auditLogRepo, responseCache, reconciliationService),
  );
  app.use("/stats", buildStatsRouter(grantRepo, proofRepo, responseCache));
  app.use("/api/stats", buildStatsRouter(grantRepo, proofRepo, responseCache));
  app.use("/proofs", buildProofsRouter(ipfsService));
  app.use("/notifications", buildNotificationsRouter(contributorRepo));
  app.use("/dashboard", buildDashboardRouter(grantRepo, milestoneRepo, proofRepo, grantSyncService));
  app.use("/analytics", buildAnalyticsRouter(grantRepo, grantViewRepo));
  app.use("/search", buildSearchRouter(dataSource));
  app.use("/profiles", buildProfilesRouter(contributorRepo, grantRepo));
  app.use("/watchlist", buildWatchlistRouter(dataSource.getRepository(UserWatchlist), grantRepo));
  app.use("/communities", buildCommunitiesRouter(communityRepo, grantRepo, activityRepo));
  app.use(buildMilestoneCommentsRouter(milestoneRepo, milestoneCommentRepo, grantReviewerRepo));
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
