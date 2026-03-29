import { Router } from "express";
import { Repository } from "typeorm";
import { Grant } from "../entities/Grant";
import { GrantSyncService } from "../services/grant-sync-service";

export const buildGrantRouter = (grantRepo: Repository<Grant>, syncService: GrantSyncService) => {
  const router = Router();

  router.get("/", async (_req, res, next) => {
    try {
      await syncService.syncAllGrants();
      const grants = await grantRepo.find({ order: { id: "ASC" } });
      res.json({ data: grants });
    } catch (error) {
      next(error);
    }
  });

  router.get("/:id", async (req, res, next) => {
    try {
      const id = Number(req.params.id);
      if (Number.isNaN(id)) {
        res.status(400).json({ error: "Invalid grant id" });
        return;
      }

      await syncService.syncGrant(id);
      const grant = await grantRepo.findOne({ where: { id } });

      if (!grant) {
        res.status(404).json({ error: "Grant not found" });
        return;
      }

      res.json({ data: grant });
    } catch (error) {
      next(error);
    }
  });

  return router;
};
