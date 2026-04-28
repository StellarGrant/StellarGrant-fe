import "reflect-metadata";
import { DataSource } from "typeorm";
import { env } from "../config/env";
import { Grant } from "../entities/Grant";
import { Milestone } from "../entities/Milestone";
import { MilestoneProof } from "../entities/MilestoneProof";
import { User } from "../entities/User";
import { GrantReviewer } from "../entities/GrantReviewer";
import { MilestoneApproval } from "../entities/MilestoneApproval";
import { Contributor } from "../entities/Contributor";
import { ReputationLog } from "../entities/ReputationLog";
import { AuditLog } from "../entities/AuditLog";
import { UserWatchlist } from "../entities/UserWatchlist";
import { Activity } from "../entities/Activity";

import { GrantView } from "../entities/GrantView";
import { ReconciliationCheckpoint } from "../entities/ReconciliationCheckpoint";
import { RateLimitLog } from "../entities/RateLimitLog";
import { Community } from "../entities/Community";
import { MilestoneComment } from "../entities/MilestoneComment";
import { GrantHistory } from "../entities/GrantHistory";
import { FeeCollection } from "../entities/FeeCollection";
import { PlatformConfig } from "../entities/PlatformConfig";
import { Report } from "../entities/Report";

export const buildDataSource = (databaseUrl = env.databaseUrl) =>
  new DataSource({
    type: databaseUrl.startsWith("sqljs") ? "sqljs" : "postgres",
    ...(databaseUrl.startsWith("sqljs")
      ? { location: databaseUrl.replace("sqljs://", ""), autoSave: false }
      : { url: databaseUrl }),
    entities: [Grant, Milestone, MilestoneProof, User, GrantReviewer, MilestoneApproval, Contributor, ReputationLog, AuditLog, UserWatchlist, Activity, GrantView, ReconciliationCheckpoint, RateLimitLog, Community, MilestoneComment, GrantHistory, FeeCollection, PlatformConfig, Report],
    synchronize: true,
  });

// Export a singleton AppDataSource for use in routes/services
export const AppDataSource = buildDataSource();
