import { DataSource, Repository } from "typeorm";
import { Grant } from "../entities/Grant";
import { Contributor } from "../entities/Contributor";
import { ReputationLog } from "../entities/ReputationLog";
import { Activity } from "../entities/Activity";
import { UserWatchlist } from "../entities/UserWatchlist";
import { SorobanContractClient } from "../soroban/types";
import { notificationService } from "./notification-service";

export class GrantSyncService {
  private readonly grantRepo: Repository<Grant>;
  private readonly contributorRepo: Repository<Contributor>;
  private readonly reputationLogRepo: Repository<ReputationLog>;
  private readonly activityRepo: Repository<Activity>;
  private readonly watchlistRepo: Repository<UserWatchlist>;

  constructor(
    private readonly dataSource: DataSource,
    private readonly sorobanClient: SorobanContractClient,
  ) {
    this.grantRepo = this.dataSource.getRepository(Grant);
    this.contributorRepo = this.dataSource.getRepository(Contributor);
    this.reputationLogRepo = this.dataSource.getRepository(ReputationLog);
    this.activityRepo = this.dataSource.getRepository(Activity);
    this.watchlistRepo = this.dataSource.getRepository(UserWatchlist);
  }

  async syncAllGrants(): Promise<void> {
    const grants = await this.sorobanClient.fetchGrants();
    for (const grant of grants) {
      const existingGrant = await this.grantRepo.findOne({ where: { id: grant.id } });
      await this.grantRepo.save(grant);
      await this.syncContributorScore(grant.recipient);

      // Log activity for new grants
      if (!existingGrant) {
        await this.logActivity({
          type: "grant_created",
          entityType: "grant",
          entityId: grant.id,
          actorAddress: grant.recipient,
          data: { title: grant.title, totalAmount: grant.totalAmount },
        });
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
    const grant = await this.sorobanClient.fetchGrantById(id);
    if (!grant) return;
    const existingGrant = await this.grantRepo.findOne({ where: { id } });
    await this.grantRepo.save(grant);
    await this.syncContributorScore(grant.recipient);

    // Log activity for new grants
    if (!existingGrant) {
      await this.logActivity({
        type: "grant_created",
        entityType: "grant",
        entityId: grant.id,
        actorAddress: grant.recipient,
        data: { title: grant.title, totalAmount: grant.totalAmount },
      });
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
