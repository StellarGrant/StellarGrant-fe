import { Router } from "express";
import { LeaderboardService } from "../services/leaderboard-service";

export const buildLeaderboardRouter = (leaderboardService: LeaderboardService) => {
  const router = Router();

  router.get("/", async (req, res, next) => {
    try {
      const period = req.query.period === "monthly" ? "monthly" : "all-time";
      const page = parseInt(String(req.query.page ?? "1"), 10);
      const limit = parseInt(String(req.query.limit ?? "20"), 10);

      const [data, total] = await leaderboardService.getLeaderboard(period, page, limit);

      res.json({
        data,
        meta: {
          total,
          page,
          limit,
          totalPages: Math.ceil(Number(total) / limit),
        },
      });
    } catch (error) {
      next(error);
    }
  });

  return router;
};
