import { Router } from "express";
import { Repository, In } from "typeorm";
import { UserWatchlist } from "../entities/UserWatchlist";
import { Grant } from "../entities/Grant";
import { Activity } from "../entities/Activity";

export const buildWatchlistRouter = (
  watchlistRepo: Repository<UserWatchlist>,
  grantRepo: Repository<Grant>,
) => {
  const router = Router();
  const activityRepo = watchlistRepo.manager.getRepository(Activity);

  router.get("/", async (req, res, next) => {
    try {
      const address = req.header("x-user-address");
      if (!address) {
        res.status(400).json({ error: "Missing x-user-address header" });
        return;
      }

      const watchlist = await watchlistRepo.find({ where: { address } });
      const grantIds = watchlist.map((w) => w.grantId);

      if (grantIds.length === 0) {
        res.json({ data: [] });
        return;
      }

      const grants = await grantRepo.find({
        where: { id: In(grantIds) },
      });
      res.json({ data: grants });
    } catch (error) {
      next(error);
    }
  });

  router.post("/:grantId", async (req, res, next) => {
    try {
      const grantId = parseInt(req.params.grantId, 10);
      const address = req.header("x-user-address");

      if (isNaN(grantId) || !address) {
        res.status(400).json({ error: "Invalid grantId or missing x-user-address" });
        return;
      }

      const grant = await grantRepo.findOne({ where: { id: grantId } });
      if (!grant) {
        res.status(404).json({ error: "Grant not found" });
        return;
      }

      await watchlistRepo.save({ address, grantId });

      await activityRepo.save({
        type: "watchlist_added" as any,
        entityType: "grant",
        entityId: grantId,
        actorAddress: address,
        data: null,
      });

      res.status(201).json({ ok: true });
    } catch (error: any) {
      if (error?.code === "23505" || error?.code === "SQLITE_CONSTRAINT") {
        res.status(409).json({ error: "Grant already in watchlist" });
        return;
      }
      next(error);
    }
  });

  router.delete("/:grantId", async (req, res, next) => {
    try {
      const grantId = parseInt(req.params.grantId, 10);
      const address = req.header("x-user-address");

      if (isNaN(grantId) || !address) {
        res.status(400).json({ error: "Invalid grantId or missing x-user-address" });
        return;
      }

      const result = await watchlistRepo.delete({ address, grantId });

      if (result.affected === 0) {
        res.status(404).json({ error: "Watchlist entry not found" });
        return;
      }

      await activityRepo.save({
        type: "watchlist_removed" as any,
        entityType: "grant",
        entityId: grantId,
        actorAddress: address,
        data: null,
      });

      res.json({ ok: true });
    } catch (error) {
      next(error);
    }
  });

  return router;
};
