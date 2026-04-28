import { Router } from "express";
import { DataSource } from "typeorm";

/**
 * Unified Search Endpoint
 * 
 * Searches across:
 * 1. Grants (title, tags, metadata)
 * 2. Contributors (address, email)
 * 3. Milestone Descriptions (description, CID)
 * 
 * Uses PostgreSQL GIN indexes for Full-Text Search when available.
 */
export const buildSearchRouter = (dataSource: DataSource) => {
  const router = Router();

  router.get("/", async (req, res, next) => {
    try {
      const query = String(req.query.q || "").trim();
      if (!query || query.length < 2) {
        return res.json({ data: [] });
      }

      const isPostgres = dataSource.options.type === "postgres";

      if (isPostgres) {
        // ---------------------------------------------------------------------
        // PostgreSQL Full-Text Search implementation
        // ---------------------------------------------------------------------
        // We use plainto_tsquery for "typo-friendly" partial matches (word-based).
        // For even better typo tolerance, pg_trgm could be used in the future.
        
        const grantsQuery = `
          SELECT 
            id::text as id, title as name, 'grant' as type,
            ts_rank(to_tsvector('english', title || ' ' || COALESCE(tags, '') || ' ' || COALESCE(CAST("localizedMetadata" AS TEXT), '')), plainto_tsquery('english', $1)) as rank
          FROM grants
          WHERE "isFlagged" = false AND to_tsvector('english', title || ' ' || COALESCE(tags, '') || ' ' || COALESCE(CAST("localizedMetadata" AS TEXT), '')) @@ plainto_tsquery('english', $1)
        `;

        const contributorsQuery = `
          SELECT 
            address as id, address as name, 'contributor' as type,
            ts_rank(to_tsvector('english', address || ' ' || COALESCE(email, '')), plainto_tsquery('english', $1)) as rank
          FROM contributors
          WHERE to_tsvector('english', address || ' ' || COALESCE(email, '')) @@ plainto_tsquery('english', $1)
        `;

        const milestonesQuery = `
          SELECT 
            id::text as id, COALESCE(description, proofCid) as name, 'milestone' as type,
            ts_rank(to_tsvector('english', COALESCE(description, '') || ' ' || proofCid), plainto_tsquery('english', $1)) as rank
          FROM milestone_proofs
          WHERE to_tsvector('english', COALESCE(description, '') || ' ' || proofCid) @@ plainto_tsquery('english', $1)
        `;

        const unifiedQuery = `
          SELECT * FROM (
            (${grantsQuery})
            UNION ALL
            (${contributorsQuery})
            UNION ALL
            (${milestonesQuery})
          ) AS search_results
          ORDER BY rank DESC
          LIMIT 50
        `;

        const results = await dataSource.query(unifiedQuery, [query]);
        res.json({ data: results });
      } else {
        // ---------------------------------------------------------------------
        // Fallback for SQLJS / SQLite (used in E2E tests)
        // ---------------------------------------------------------------------
        const queryLike = `%${query.toLowerCase()}%`;
        
        const grants = await dataSource.query(
          `SELECT id, title as name, 'grant' as type FROM grants WHERE "isFlagged" = 0 AND (LOWER(title) LIKE ? OR LOWER(tags) LIKE ?)`,
          [queryLike, queryLike]
        );
        
        const contributors = await dataSource.query(
          `SELECT address as id, address as name, 'contributor' as type FROM contributors WHERE LOWER(address) LIKE ? OR LOWER(email) LIKE ?`,
          [queryLike, queryLike]
        );

        const milestones = await dataSource.query(
          `SELECT id, COALESCE(description, proofCid) as name, 'milestone' as type FROM milestone_proofs WHERE LOWER(description) LIKE ? OR LOWER(proofCid) LIKE ?`,
          [queryLike, queryLike]
        );

        const results = [...grants, ...contributors, ...milestones];
        res.json({ data: results.slice(0, 50) });
      }
    } catch (error) {
      next(error);
    }
  });

  return router;
};
