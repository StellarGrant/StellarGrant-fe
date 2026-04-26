import { Router } from "express";
import { Repository } from "typeorm";
import { z } from "zod";
import { GrantSyncService } from "../services/grant-sync-service";
import { ReconciliationService } from "../services/reconciliation-service";
import { Contributor } from "../entities/Contributor";
import { AuditLog } from "../entities/AuditLog";
import { Grant } from "../entities/Grant";
import { PlatformConfig } from "../entities/PlatformConfig";
import { FeeCollection } from "../entities/FeeCollection";

const VALID_BULK_ACTIONS = ["approve", "reject", "flag"] as const;
type BulkAction = (typeof VALID_BULK_ACTIONS)[number];

const ACTION_STATUS_MAP: Record<BulkAction, string> = {
  approve: "approved",
  reject: "rejected",
  flag: "flagged",
};

const bulkSchema = z.object({
  grantIds: z.array(z.number().int().positive()).min(1).max(100),
  action: z.enum(VALID_BULK_ACTIONS),
});

const configSchema = z.object({
  feePercentage: z.number().min(0).max(100),
});

export const buildAdminRouter = (
  grantSyncService: GrantSyncService,
  contributorRepo: Repository<Contributor>,
  auditLogRepo: Repository<AuditLog>,
  reconciliationService?: ReconciliationService,
) => {
  const router = Router();
  const grantRepo: Repository<Grant> = auditLogRepo.manager.getRepository(Grant);
  const configRepo = auditLogRepo.manager.getRepository(PlatformConfig);
  const feeRepo = auditLogRepo.manager.getRepository(FeeCollection);

  router.post("/sync/:grant_id", async (req, res, next) => {
    try {
      const grantId = parseInt(req.params.grant_id, 10);
      if (isNaN(grantId)) {
        res.status(400).json({ error: "Invalid grant ID" });
        return;
      }

      await grantSyncService.syncGrant(grantId);

      await auditLogRepo.save({
        adminAddress: (req as any).adminAddress,
        action: "SYNC_GRANT",
        target: `grant:${grantId}`,
        details: `Force synced grant ${grantId}`,
      });

      res.json({ ok: true, message: `Grant ${grantId} synced` });
    } catch (error) {
      next(error);
    }
  });

  router.patch("/users/:address/blacklist", async (req, res, next) => {
    try {
      const { address } = req.params;
      const { blacklist } = req.body;

      if (typeof blacklist !== "boolean") {
        res.status(400).json({ error: "Missing or invalid 'blacklist' field (boolean)" });
        return;
      }

      let contributor = await contributorRepo.findOne({ where: { address } });
      if (!contributor) {
        contributor = contributorRepo.create({ address });
      }

      contributor.isBlacklisted = blacklist;
      await contributorRepo.save(contributor);

      await auditLogRepo.save({
        adminAddress: (req as any).adminAddress,
        action: blacklist ? "BLACKLIST_USER" : "UNBLACKLIST_USER",
        target: `user:${address}`,
        details: `${blacklist ? "Blacklisted" : "Unblacklisted"} user ${address}`,
      });

      res.json({ ok: true, isBlacklisted: blacklist });
    } catch (error) {
      next(error);
    }
  });

  /**
   * POST /admin/grants/bulk
   * Bulk approve, reject, or flag multiple grants atomically.
   * Body: { grantIds: number[], action: "approve" | "reject" | "flag" }
   */
  router.post("/grants/bulk", async (req, res, next) => {
    try {
      const parsed = bulkSchema.safeParse(req.body);
      if (!parsed.success) {
        res.status(400).json({ error: "Invalid payload", details: parsed.error.issues });
        return;
      }

      const { grantIds, action } = parsed.data;
      const newStatus = ACTION_STATUS_MAP[action];
      const adminAddress: string = (req as any).adminAddress;

      const results: { id: number; updated: boolean }[] = [];
      const invalidIds: number[] = [];

      await grantRepo.manager.transaction(async (em) => {
        const grants = await em.findByIds(Grant, grantIds);
        const foundIds = new Set(grants.map((g) => g.id));

        for (const id of grantIds) {
          if (!foundIds.has(id)) {
            invalidIds.push(id);
          }
        }

        if (invalidIds.length > 0) {
          throw Object.assign(
            new Error(`Grant IDs not found: ${invalidIds.join(", ")}`),
            { status: 404 },
          );
        }

        for (const grant of grants) {
          grant.status = newStatus;
          results.push({ id: grant.id, updated: true });
        }
        await em.save(grants);

        await em.save(AuditLog, {
          adminAddress,
          action: `BULK_${action.toUpperCase()}`,
          target: `grants:${grantIds.join(",")}`,
          details: JSON.stringify({ grantIds, newStatus }),
        });
      });

      res.json({
        data: {
          action,
          newStatus,
          affected: results.length,
          results,
        },
      });
    } catch (error: any) {
      if (error?.status === 404) {
        res.status(404).json({ error: error.message });
        return;
      }
      next(error);
    }
  });

  /**
   * POST /admin/reconcile
   * Manually trigger a reconciliation run. Returns the result immediately.
   */
  router.post("/reconcile", async (req, res, next) => {
    if (!reconciliationService) {
      res.status(503).json({ error: "Reconciliation service not available" });
      return;
    }
    try {
      const result = await reconciliationService.run();

      await auditLogRepo.save({
        adminAddress: (req as any).adminAddress,
        action: "TRIGGER_RECONCILIATION",
        target: `ledgers:${result.fromLedger}-${result.toLedger}`,
        details: JSON.stringify(result),
      });

      res.json({ ok: true, result });
    } catch (error) {
      next(error);
    }
  });

  /**
   * POST /admin/reconcile
   * Manually trigger a reconciliation run. Returns the result immediately.
   */
  router.post("/reconcile", async (req, res, next) => {
    if (!reconciliationService) {
      res.status(503).json({ error: "Reconciliation service not available" });
      return;
    }
    try {
      const result = await reconciliationService.run();

      await auditLogRepo.save({
        adminAddress: (req as any).adminAddress,
        action: "TRIGGER_RECONCILIATION",
        target: `ledgers:${result.fromLedger}-${result.toLedger}`,
        details: JSON.stringify(result),
      });

      res.json({ ok: true, result });
    } catch (error) {
      next(error);
    }
  });

  return router;
};
