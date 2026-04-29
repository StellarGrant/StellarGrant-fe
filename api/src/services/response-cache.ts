import Redis from "ioredis";
import { env } from "../config/env";

const CACHE_PREFIX = "sg:cache:v1:";

export const responseCacheKeys = {
  stats: () => `${CACHE_PREFIX}stats`,
  grantsFirstPage: (lang: string) => `${CACHE_PREFIX}grants:first:${lang}`,
};

/**
 * Redis-backed HTTP response cache for public, unpersonalized endpoints.
 * When REDIS_URL is unset, all operations are no-ops and reads always miss.
 */
export class ResponseCacheService {
  private redis: Redis | null = null;

  constructor(redisUrl: string) {
    if (!redisUrl.trim()) return;
    this.redis = new Redis(redisUrl, {
      maxRetriesPerRequest: 2,
      lazyConnect: true,
      enableOfflineQueue: false,
    });
    this.redis.on("error", () => {
      // Errors are handled per-command; avoid crashing the process on transient outages.
    });
  }

  isEnabled(): boolean {
    return this.redis !== null;
  }

  async get(key: string): Promise<string | null> {
    if (!this.redis) return null;
    try {
      if (this.redis.status === "wait") await this.redis.connect();
      return await this.redis.get(key);
    } catch {
      return null;
    }
  }

  async set(key: string, value: string): Promise<void> {
    if (!this.redis) return;
    try {
      if (this.redis.status === "wait") await this.redis.connect();
      await this.redis.setex(key, env.cacheTtlSeconds, value);
    } catch {
      // ignore cache write failures
    }
  }

  /** Clears aggregated stats and grant list caches (e.g. after grant or milestone updates). */
  async invalidateGrantsAndStats(): Promise<void> {
    if (!this.redis) return;
    try {
      if (this.redis.status === "wait") await this.redis.connect();
      const keys = new Set<string>();
      let cursor = "0";
      do {
        const [next, found] = await this.redis.scan(
          cursor,
          "MATCH",
          `${CACHE_PREFIX}*`,
          "COUNT",
          128,
        );
        cursor = next;
        for (const k of found) keys.add(k);
      } while (cursor !== "0");
      const list = [...keys];
      if (list.length > 0) {
        await this.redis.del(...list);
      }
    } catch {
      // ignore invalidation failures
    }
  }
}
