import { Router } from "express";
import { Repository } from "typeorm";
import { Activity } from "../entities/Activity";
import { Contributor } from "../entities/Contributor";

export const buildActivityRouter = (
  activityRepo: Repository<Activity>,
  contributorRepo: Repository<Contributor>,
) => {
  const router = Router();

  router.get("/", async (req, res, next) => {
    try {
      const limit = Math.min(Number(req.query.limit) || 20, 100);
      const cursor = req.query.cursor ? String(req.query.cursor) : null;

      const qb = activityRepo.createQueryBuilder("activity")
        .orderBy("activity.timestamp", "DESC")
        .addOrderBy("activity.id", "DESC")
        .limit(limit + 1); // Fetch one extra to determine if there's a next page

      if (cursor) {
        // Cursor format: "timestamp:id"
        const [timestamp, id] = cursor.split(":");
        qb.andWhere(
          "(activity.timestamp < :timestamp OR (activity.timestamp = :timestamp AND activity.id < :id))",
          { timestamp: new Date(timestamp), id: Number(id) }
        );
      }

      const activities = await qb.getMany();
      const addresses = [...new Set(activities.map((a) => a.actorAddress).filter(Boolean))] as string[];
      const contributors = addresses.length
        ? await contributorRepo.findBy(addresses.map((address) => ({ address })))
        : [];
      const byAddress = new Map(contributors.map((c) => [c.address, c]));

      let nextCursor: string | null = null;
      if (activities.length > limit) {
        // Remove the extra item
        activities.pop();
        const lastActivity = activities[activities.length - 1];
        nextCursor = `${lastActivity.timestamp.toISOString()}:${lastActivity.id}`;
      }

      res.json({
        data: activities.map((a) => ({
          ...a,
          actorProfile: a.actorAddress ? {
            address: a.actorAddress,
            bio: byAddress.get(a.actorAddress)?.bio ?? null,
            profilePictureUrl: byAddress.get(a.actorAddress)?.profilePictureUrl ?? null,
            githubUrl: byAddress.get(a.actorAddress)?.githubUrl ?? null,
            twitterUrl: byAddress.get(a.actorAddress)?.twitterUrl ?? null,
            linkedinUrl: byAddress.get(a.actorAddress)?.linkedinUrl ?? null,
            updatedAt: byAddress.get(a.actorAddress)?.updatedAt ?? null,
          } : null,
        })),
        meta: {
          nextCursor,
          limit,
        },
      });
    } catch (error) {
      next(error);
    }
  });

  return router;
};
