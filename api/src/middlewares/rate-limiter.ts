import rateLimit from "express-rate-limit";
import { DataSource } from "typeorm";
import { RateLimitLog } from "../entities/RateLimitLog";

export const createRateLimiter = (dataSource: DataSource) => {
  const repo = dataSource.getRepository(RateLimitLog);

  return rateLimit({
    windowMs: 60 * 1000,
    max: 60,

    handler: async (req, res) => {
      // log blocked request
      await repo.save({
        ip: req.ip,
        path: req.originalUrl,
        method: req.method,
        userAgent: req.headers["user-agent"] || "",
      });

      res.status(429).json({
        error: "Too many requests",
      });
    },
  });
};