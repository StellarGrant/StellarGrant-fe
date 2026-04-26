import "reflect-metadata";
import { DataSource } from "typeorm";
import { env } from "../config/env";
import { Grant } from "../entities/Grant";
import { MilestoneProof } from "../entities/MilestoneProof";
import { Contributor } from "../entities/Contributor";
import { ReputationLog } from "../entities/ReputationLog";
import { AuditLog } from "../entities/AuditLog";
import { UserWatchlist } from "../entities/UserWatchlist";
import { Activity } from "../entities/Activity";
import { GrantView } from "../entities/GrantView";
import { PlatformConfig } from "../entities/PlatformConfig";
import { FeeCollection } from "../entities/FeeCollection";

export const buildDataSource = (databaseUrl = env.databaseUrl) =>
  new DataSource({
    type: databaseUrl.startsWith("sqljs") ? "sqljs" : "postgres",
    ...(databaseUrl.startsWith("sqljs")
      ? { location: databaseUrl.replace("sqljs://", ""), autoSave: false }
      : { url: databaseUrl }),
    entities: [
      Grant,
      MilestoneProof,
      Contributor,
      ReputationLog,
      AuditLog,
      UserWatchlist,
      Activity,
      GrantView,
      PlatformConfig,
      FeeCollection,
    ],
    synchronize: true,
  });
