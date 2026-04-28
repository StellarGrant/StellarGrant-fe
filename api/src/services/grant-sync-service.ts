import { DataSource, In, Repository } from "typeorm";
import { Grant } from "../entities/Grant";
import { Contributor } from "../entities/Contributor";
import { ReputationLog } from "../entities/ReputationLog";
import { Activity } from "../entities/Activity";
import { UserWatchlist } from "../entities/UserWatchlist";
import { Milestone } from "../entities/Milestone";
import { SorobanContractClient, SorobanMilestone } from "../soroban/types";
import { notificationService } from "./notification-service";
import { metricsService } from "./metrics-service";
import { GrantHistory } from "../entities/GrantHistory";
import { computeDiff } from "../utils/diff";

export class GrantSyncService {
  private readonly grantRepo: Repository<Grant>;
  private readonly contributorRepo: Repository<Contributor>;
  private readonly reputationLogRepo: Repository<ReputationLog>;
  private readonly activityRepo: Repository<Activity>;
  private readonly watchlistRepo: Repository<UserWatchlist>;
  private readonly milestoneRepo: Repository<Milestone>;
  private readonly grantHistoryRepo: Repository<GrantHistory>;

  constructor(
    private readonly dataSource: DataSource,
    private readonly sorobanClient: SorobanContractClient,
    private readonly onInvalidatePublicCaches?: () => void | Promise<void>,
  ) {
    this.grantRepo = this.dataSource.getRepository(Grant);
    this.contributorRepo = this.dataSource.getRepository(Contributor);
    this.reputationLogRepo = this.dataSource.getRepository(ReputationLog);
    this.activityRepo = this.dataSource.getRepository(Activity);
    this.watchlistRepo = this.dataSource.getRepository(UserWatchlist);
    this.milestoneRepo = this.dataSource.getRepository(Milestone);
    this.grantHistoryRepo = this.dataSource.getRepository(GrantHistory);
  }

  async syncAllGrants(): Promise<void> {
    await this.onInvalidatePublicCaches?.();
    const grants = await this.sorobanClient.fetchGrants();
    for (const grant of grants) {
      const { milestones, ...grantRecord } = grant;
      const existingGrant = await this.grantRepo.findOne({ where: { id: grant.id } });
      await this.grantRepo.save(grantRecord);
      await this.syncMilestones(grant.id, milestones);
      await this.syncContributorScore(grant.recipient);

      if (existingGrant) {
        const diff = computeDiff(existingGrant, grantRecord);
        if (Object.keys(diff).length > 0) {
          await this.grantHistoryRepo.save({
            grantId: grant.id,
            snapshot: grantRecord,
            diff,
          });
        }
      } else {
        await this.grantHistoryRepo.save({
          grantId: grant.id,
          snapshot: grantRecord,
          diff: computeDiff({}, grantRecord),
        });
      }

      // Log activity for new grants
      if (!existingGrant) {
        await this.logActivity({
          type: "grant_created",
          entityType: "grant",
          entityId: grant.id,
          actorAddress: grant.recipient,
          data: { title: grant.title, totalAmount: grant.totalAmount },
        });
        metricsService.incrementGrantCreated();
        notificationService.notifyUser(grant.recipient, "grant_created", { title: grant.title, grantId: grant.id });
      } else if (existingGrant.status !== grant.status) {
        // Log activity for status changes
        await this.logActivity({
          type: "grant_updated",
          entityType: "grant",
          entityId: grant.id,
          actorAddress: grant.recipient,
          data: { oldStatus: existingGrant.status, newStatus: grant.status },
        });
        notificationService.notifyUser(grant.recipient, "grant_updated", { 
          grantId: grant.id, 
          title: grant.title,
          oldStatus: existingGrant.status, 
          newStatus: grant.status 
        });
        await this.notifyWatchers(grant.id, "grant_updated", {
          grantId: grant.id,
          title: grant.title,
          oldStatus: existingGrant.status,
          newStatus: grant.status,
        });
      }
    }
  }

  async syncGrant(id: number): Promise<void> {
    await this.onInvalidatePublicCaches?.();
    const grant = await this.sorobanClient.fetchGrantById(id);
    if (!grant) return;
    const { milestones, ...grantRecord } = grant;
    const existingGrant = await this.grantRepo.findOne({ where: { id } });
    await this.grantRepo.save(grantRecord);
    await this.syncMilestones(grant.id, milestones);
    await this.syncContributorScore(grant.recipient);

    if (existingGrant) {
      const diff = computeDiff(existingGrant, grantRecord);
      if (Object.keys(diff).length > 0) {
        await this.grantHistoryRepo.save({
          grantId: grant.id,
          snapshot: grantRecord,
          diff,
        });
      }
    } else {
      await this.grantHistoryRepo.save({
        grantId: grant.id,
        snapshot: grantRecord,
        diff: computeDiff({}, grantRecord),
      });
    }

    // Log activity for new grants
    if (!existingGrant) {
      await this.logActivity({
        type: "grant_created",
        entityType: "grant",
        entityId: grant.id,
        actorAddress: grant.recipient,
        data: { title: grant.title, totalAmount: grant.totalAmount },
      });
      metricsService.incrementGrantCreated();
      notificationService.notifyUser(grant.recipient, "grant_created", { title: grant.title, grantId: grant.id });
    } else if (existingGrant.status !== grant.status) {
      // Log activity for status changes
      await this.logActivity({
        type: "grant_updated",
        entityType: "grant",
        entityId: grant.id,
        actorAddress: grant.recipient,
        data: { oldStatus: existingGrant.status, newStatus: grant.status },
      });
      notificationService.notifyUser(grant.recipient, "grant_updated", { 
        grantId: grant.id, 
        title: grant.title,
        oldStatus: existingGrant.status, 
        newStatus: grant.status 
      });
      await this.notifyWatchers(grant.id, "grant_updated", {
        grantId: grant.id,
        title: grant.title,
        oldStatus: existingGrant.status,
        newStatus: grant.status,
      });
    }
  }

  private async notifyWatchers(grantId: number, type: string, data: any): Promise<void> {
    const watchers = await this.watchlistRepo.find({ where: { grantId } });
    for (const watcher of watchers) {
      notificationService.notifyUser(watcher.address, type as any, data);
    }
  }

  private async syncMilestones(grantId: number, milestones?: SorobanMilestone[]): Promise<void> {
    if (!milestones) {
      return;
    }

    const existing = await this.milestoneRepo.find({ where: { grantId } });
    const existingByIdx = new Map(existing.map((milestone) => [milestone.idx, milestone]));
    const nextIdxs = new Set<number>();

    for (const milestone of milestones) {
      nextIdxs.add(milestone.idx);
      const previous = existingByIdx.get(milestone.idx);
      const deadlineChanged = previous?.deadline !== milestone.deadline;

      await this.milestoneRepo.save({
        id: previous?.id,
        grantId,
        idx: milestone.idx,
        title: milestone.title,
        description: milestone.description ?? null,
        deadline: milestone.deadline,
        lastDeadlineReminderAt: deadlineChanged ? null : previous?.lastDeadlineReminderAt ?? null,
        lastDeadlineReminderDaysBefore: deadlineChanged ? null : previous?.lastDeadlineReminderDaysBefore ?? null,
        overdueNotifiedAt: deadlineChanged ? null : previous?.overdueNotifiedAt ?? null,
      });
    }

    const staleIds = existing
      .filter((milestone) => !nextIdxs.has(milestone.idx))
      .map((milestone) => milestone.id);

    if (staleIds.length > 0) {
      await this.milestoneRepo.delete({ id: In(staleIds) });
    }
  }

  private async syncContributorScore(address: string): Promise<void> {
    const score = await this.sorobanClient.fetchContributorScore(address);
    if (!score) return;

    let contributor = await this.contributorRepo.findOne({ where: { address } });
    const oldReputation = contributor?.reputation ?? 0;

    if (!contributor) {
      contributor = new Contributor();
      contributor.address = address;
    }

    // Count completed grants for this recipient
    const totalGrantsCompleted = await this.grantRepo.count({
      where: { recipient: address, status: "completed" }
    });

    contributor.reputation = score.reputation;
    contributor.totalGrantsCompleted = totalGrantsCompleted;
    await this.contributorRepo.save(contributor);

    // If reputation increased, log it for monthly leaderboard
    if (score.reputation > oldReputation) {
      const log = new ReputationLog();
      log.address = address;
      log.gain = score.reputation - oldReputation;
      await this.reputationLogRepo.save(log);

      // Log activity for reputation gain
      await this.logActivity({
        type: "reputation_gained",
        entityType: "contributor",
        entityId: null,
        actorAddress: address,
        data: { gain: score.reputation - oldReputation, newReputation: score.reputation },
      });
    }
  }

  private async logActivity(params: {
    type: string;
    entityType: string;
    entityId: number | null;
    actorAddress: string | null;
    data: Record<string, unknown> | null;
  }): Promise<void> {
    const activity = new Activity();
    activity.type = params.type as any;
    activity.entityType = params.entityType as any;
    activity.entityId = params.entityId;
    activity.actorAddress = params.actorAddress;
    activity.data = params.data;
    await this.activityRepo.save(activity);
  }
}
