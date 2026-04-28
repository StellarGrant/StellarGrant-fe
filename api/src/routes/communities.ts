import { Router } from "express";
import { Repository } from "typeorm";
import { z } from "zod";
import { Community } from "../entities/Community";
import { Grant } from "../entities/Grant";
import { env } from "../config/env";
import { Activity } from "../entities/Activity";

const createCommunitySchema = z.object({
  name: z.string().min(2).max(120),
  description: z.string().max(2000).optional(),
  logoUrl: z.string().url().max(2000).optional(),
  adminAddresses: z.array(z.string().min(10).max(120)).max(20).optional(),
  featured: z.boolean().optional(),
});

const updateCommunitySchema = z.object({
  description: z.string().max(2000).optional(),
  logoUrl: z.string().url().max(2000).nullable().optional(),
  featured: z.boolean().optional(),
});

const isPlatformAdmin = (address?: string) =>
  !!address && env.adminAddresses.includes(address);

export const buildCommunitiesRouter = (
  communityRepo: Repository<Community>,
  grantRepo: Repository<Grant>,
  activityRepo: Repository<Activity>,
) => {
  const router = Router();

  router.get("/", async (_req, res, next) => {
    try {
      const communities = await communityRepo.find({ order: { featured: "DESC", name: "ASC" } });
      res.json({ data: communities });
    } catch (error) {
      next(error);
    }
  });

  router.post("/", async (req, res, next) => {
    try {
      const adminAddress = req.header("x-admin-address") ?? undefined;
      if (!isPlatformAdmin(adminAddress)) {
        res.status(403).json({ error: "Admin privileges required" });
        return;
      }

      const parsed = createCommunitySchema.safeParse(req.body);
      if (!parsed.success) {
        res.status(400).json({ error: "Invalid payload", details: parsed.error.issues });
        return;
      }

      const fallbackAdmin = adminAddress as string;
      const payload = parsed.data;
      const created = await communityRepo.save({
        name: payload.name.trim(),
        description: payload.description?.trim() ?? null,
        logoUrl: payload.logoUrl ?? null,
        adminAddresses: payload.adminAddresses?.map((address) => address.trim()) ?? [fallbackAdmin],
        featured: payload.featured ?? false,
      });

      res.status(201).json({ data: created });
    } catch (error: any) {
      if (error?.code === "23505" || error?.code === "SQLITE_CONSTRAINT") {
        res.status(409).json({ error: "Community name already exists" });
        return;
      }
      next(error);
    }
  });

  router.patch("/:id", async (req, res, next) => {
    try {
      const id = Number(req.params.id);
      if (Number.isNaN(id)) {
        res.status(400).json({ error: "Invalid community id" });
        return;
      }

      const community = await communityRepo.findOne({ where: { id } });
      if (!community) {
        res.status(404).json({ error: "Community not found" });
        return;
      }

      const actor = req.header("x-admin-address") ?? undefined;
      const canManage = isPlatformAdmin(actor) || (!!actor && (community.adminAddresses ?? []).includes(actor));
      if (!canManage) {
        res.status(403).json({ error: "Community admin privileges required" });
        return;
      }

      const parsed = updateCommunitySchema.safeParse(req.body);
      if (!parsed.success) {
        res.status(400).json({ error: "Invalid payload", details: parsed.error.issues });
        return;
      }

      const payload = parsed.data;
      if (payload.description !== undefined) {
        community.description = payload.description.trim();
      }
      if (payload.logoUrl !== undefined) {
        community.logoUrl = payload.logoUrl;
      }
      if (payload.featured !== undefined) {
        community.featured = payload.featured;
      }

      const saved = await communityRepo.save(community);
      res.json({ data: saved });
    } catch (error) {
      next(error);
    }
  });

  router.get("/:id/grants", async (req, res, next) => {
    try {
      const id = Number(req.params.id);
      if (Number.isNaN(id)) {
        res.status(400).json({ error: "Invalid community id" });
        return;
      }

      const community = await communityRepo.findOne({ where: { id } });
      if (!community) {
        res.status(404).json({ error: "Community not found" });
        return;
      }

      const grants = await grantRepo.find({
        where: { communityId: id },
        order: { updatedAt: "DESC" },
      });

      const grantIds = grants.map((grant) => grant.id);
      const activity = grantIds.length
        ? await activityRepo.find({
            where: grantIds.map((grantId) => ({ entityType: "grant", entityId: grantId })),
            order: { timestamp: "DESC" },
            take: 100,
          })
        : [];

      res.json({ data: grants, community, activity });
    } catch (error) {
      next(error);
    }
  });

  router.post("/:id/grants/:grantId", async (req, res, next) => {
    try {
      const communityId = Number(req.params.id);
      const grantId = Number(req.params.grantId);
      if (Number.isNaN(communityId) || Number.isNaN(grantId)) {
        res.status(400).json({ error: "Invalid id" });
        return;
      }

      const actor = req.header("x-admin-address") ?? undefined;
      if (!isPlatformAdmin(actor)) {
        res.status(403).json({ error: "Admin privileges required" });
        return;
      }

      const community = await communityRepo.findOne({ where: { id: communityId } });
      if (!community) {
        res.status(404).json({ error: "Community not found" });
        return;
      }

      const grant = await grantRepo.findOne({ where: { id: grantId } });
      if (!grant) {
        res.status(404).json({ error: "Grant not found" });
        return;
      }

      grant.communityId = communityId;
      await grantRepo.save(grant);
      res.json({ data: grant });
    } catch (error) {
      next(error);
    }
  });

  return router;
};
