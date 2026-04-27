import { Keypair } from "@stellar/stellar-sdk";
import request from "supertest";
import { beforeAll, afterAll, describe, expect, it } from "vitest";
import { DataSource } from "typeorm";
import { createApp } from "../../src/app";
import { buildDataSource } from "../../src/db/data-source";
import { SignatureService } from "../../src/services/signature-service";
import { MockSorobanContractClient } from "../../src/soroban/mock-client";

describe("API e2e", () => {
  let dataSource: DataSource;
  const sorobanClient = new MockSorobanContractClient();
  const signatureService = new SignatureService();

  beforeAll(async () => {
    dataSource = buildDataSource("sqljs://memory");
    await dataSource.initialize();
  });

  afterAll(async () => {
    if (dataSource?.isInitialized) {
      await dataSource.destroy();
    }
  });

  // -------------------------------------------------------------------------
  // Basic list
  // -------------------------------------------------------------------------

  it("fetches grants from mock Soroban and caches them", async () => {
    const app = createApp(dataSource, sorobanClient);
    const response = await request(app).get("/grants");

    expect(response.status).toBe(200);
    // 4 mock grants total
    expect(response.body.data).toHaveLength(4);
    expect(response.body.data[0].title).toBe("Open Source Grants Q2");
  });

  it("returns the same protocol stats from /stats and /api/stats", async () => {
    const app = createApp(dataSource, sorobanClient);
    await request(app).get("/grants");
    const a = await request(app).get("/stats");
    const b = await request(app).get("/api/stats");
    expect(a.status).toBe(200);
    expect(b.status).toBe(200);
    expect(a.body).toEqual(b.body);
    expect(a.body.totalGrants).toBe(4);
    expect(typeof a.body.totalFunded).toBe("number");
    expect(typeof a.body.milestonesCompleted).toBe("number");
  });

  it("returns pagination meta with total, page, limit, totalPages", async () => {
    const app = createApp(dataSource, sorobanClient);
    const response = await request(app).get("/grants");

    expect(response.status).toBe(200);
    const { meta } = response.body;
    expect(meta).toBeDefined();
    expect(meta.total).toBe(4);
    expect(meta.page).toBe(1);
    expect(meta.limit).toBe(20);
    expect(meta.totalPages).toBe(1);
  });

  // -------------------------------------------------------------------------
  // Pagination
  // -------------------------------------------------------------------------

  describe("pagination", () => {
    it("respects page and limit params", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?page=1&limit=2");

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(2);
      expect(response.body.meta.total).toBe(4);
      expect(response.body.meta.totalPages).toBe(2);
      expect(response.body.meta.page).toBe(1);
      expect(response.body.meta.limit).toBe(2);
    });

    it("returns the second page correctly", async () => {
      const app = createApp(dataSource, sorobanClient);
      const page1 = await request(app).get("/grants?page=1&limit=2");
      const page2 = await request(app).get("/grants?page=2&limit=2");

      expect(page2.status).toBe(200);
      expect(page2.body.data).toHaveLength(2);

      // Pages should not overlap
      const ids1 = page1.body.data.map((g: { id: number }) => g.id);
      const ids2 = page2.body.data.map((g: { id: number }) => g.id);
      const overlap = ids1.filter((id: number) => ids2.includes(id));
      expect(overlap).toHaveLength(0);
    });

    it("returns an empty page beyond the last page", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?page=99&limit=20");

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(0);
      expect(response.body.meta.total).toBe(4);
    });

    it("rejects invalid page values", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?page=0");
      expect(response.status).toBe(400);
    });

    it("rejects limit > 100", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?limit=101");
      expect(response.status).toBe(400);
    });

    it("rejects limit < 1", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?limit=0");
      expect(response.status).toBe(400);
    });
  });

  // -------------------------------------------------------------------------
  // Status filter
  // -------------------------------------------------------------------------

  describe("status filter", () => {
    it("filters by status=active (exact, case-insensitive)", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?status=active");

      expect(response.status).toBe(200);
      // Grants #1 and #3 are active
      expect(response.body.data).toHaveLength(2);
      response.body.data.forEach((g: { status: string }) => {
        expect(g.status.toLowerCase()).toBe("active");
      });
    });

    it("filters by status=ACTIVE (uppercase)", async () => {
      const app = createApp(dataSource, sorobanClient);
      const res = await request(app).get("/grants?status=ACTIVE");
      expect(res.status).toBe(200);
      expect(res.body.data).toHaveLength(2);
    });

    it("filters by status=review", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?status=review");

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(1);
      expect(response.body.data[0].id).toBe(2);
    });

    it("filters by status=pending", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?status=pending");

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(1);
      expect(response.body.data[0].id).toBe(4);
    });

    it("returns zero results for a non-existent status", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?status=nonexistent");
      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(0);
      expect(response.body.meta.total).toBe(0);
    });
  });

  // -------------------------------------------------------------------------
  // Tag filter
  // -------------------------------------------------------------------------

  describe("tags filter", () => {
    it("filters by a single tag (comma-separated query param)", async () => {
      const app = createApp(dataSource, sorobanClient);
      // Grants #1 and #2 both have open-source
      const response = await request(app).get("/grants?tags=open-source");

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(2);
    });

    it("filters by a tag using repeated params", async () => {
      const app = createApp(dataSource, sorobanClient);
      // web3 appears in grants #1 and #3
      const response = await request(app).get("/grants?tags=web3");

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(2);
    });

    it("narrows results when multiple tags are supplied", async () => {
      const app = createApp(dataSource, sorobanClient);
      // Only grant #1 has BOTH open-source AND web3
      const response = await request(app).get(
        "/grants?tags=open-source&tags=web3",
      );

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(1);
      expect(response.body.data[0].id).toBe(1);
    });

    it("returns zero results when no grants match the tag", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?tags=nonexistenttag");
      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(0);
    });
  });

  // -------------------------------------------------------------------------
  // Funder (recipient) filter
  // -------------------------------------------------------------------------

  describe("funder filter", () => {
    it("filters by partial funder/recipient address", async () => {
      const app = createApp(dataSource, sorobanClient);
      // Grant #1's recipient starts with GBRP
      const response = await request(app).get("/grants?funder=GBRP");

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(1);
      expect(response.body.data[0].id).toBe(1);
    });
  });

  // -------------------------------------------------------------------------
  // Combined filters
  // -------------------------------------------------------------------------

  describe("combined filters and pagination", () => {
    it("combines status and tag filters", async () => {
      const app = createApp(dataSource, sorobanClient);
      // active grants that also have web3: #1 and #3
      const response = await request(app).get(
        "/grants?status=active&tags=web3",
      );

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(2);
      response.body.data.forEach((g: { status: string }) => {
        expect(g.status.toLowerCase()).toBe("active");
      });
    });

    it("combines filters with pagination", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get(
        "/grants?status=active&page=1&limit=1",
      );

      expect(response.status).toBe(200);
      expect(response.body.data).toHaveLength(1);
      expect(response.body.meta.total).toBe(2);
      expect(response.body.meta.totalPages).toBe(2);
    });
  });

  // -------------------------------------------------------------------------
  // Sorting
  // -------------------------------------------------------------------------

  describe("sorting", () => {
    it("sorts by totalAmount ASC", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get(
        "/grants?sortBy=totalAmount&order=ASC",
      );

      expect(response.status).toBe(200);
      const amounts = response.body.data.map((g: { totalAmount: string }) =>
        BigInt(g.totalAmount),
      );
      for (let i = 1; i < amounts.length; i++) {
        expect(amounts[i - 1] <= amounts[i]).toBe(true);
      }
    });

    it("sorts by totalAmount DESC", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get(
        "/grants?sortBy=totalAmount&order=DESC",
      );

      expect(response.status).toBe(200);
      const amounts = response.body.data.map((g: { totalAmount: string }) =>
        BigInt(g.totalAmount),
      );
      for (let i = 1; i < amounts.length; i++) {
        expect(amounts[i - 1] >= amounts[i]).toBe(true);
      }
    });

    it("falls back to id ASC for unknown sortBy values", async () => {
      const app = createApp(dataSource, sorobanClient);
      const response = await request(app).get("/grants?sortBy=invalid");

      expect(response.status).toBe(200);
      const ids = response.body.data.map((g: { id: number }) => g.id);
      const sorted = [...ids].sort((a, b) => a - b);
      expect(ids).toEqual(sorted);
    });
  });

  // -------------------------------------------------------------------------
  // Single grant by ID
  // -------------------------------------------------------------------------

  it("returns a specific grant by id", async () => {
    const app = createApp(dataSource, sorobanClient);
    const response = await request(app).get("/grants/1");

    expect(response.status).toBe(200);
    expect(response.body.data.id).toBe(1);
  });

  it("includes milestone deadline summary in the grant list response", async () => {
    const app = createApp(dataSource, sorobanClient);
    const response = await request(app).get("/grants");

    expect(response.status).toBe(200);
    const grant = response.body.data.find((item: { id: number }) => item.id === 1);
    expect(grant.milestoneSummary).toMatchObject({
      total: 2,
      submitted: 0,
      overdue: 1,
      upcoming: 1,
    });
    expect(grant.hasOverdueMilestones).toBe(true);
  });

  it("flags overdue milestones in the single-grant response", async () => {
    const app = createApp(dataSource, sorobanClient);
    const response = await request(app).get("/grants/1");

    expect(response.status).toBe(200);
    expect(response.body.data.milestones).toHaveLength(2);

    const overdueMilestone = response.body.data.milestones.find((milestone: { idx: number }) => milestone.idx === 1);
    const upcomingMilestone = response.body.data.milestones.find((milestone: { idx: number }) => milestone.idx === 0);

    expect(overdueMilestone.overdue).toBe(true);
    expect(overdueMilestone.submitted).toBe(false);
    expect(upcomingMilestone.overdue).toBe(false);
    expect(response.body.data.hasOverdueMilestones).toBe(true);
  });

  it("returns upcoming and overdue milestones on the dashboard endpoint", async () => {
    const app = createApp(dataSource, sorobanClient);
    const response = await request(app).get(
      "/dashboard/GBRPYHIL2C2WBO36G6UIGR2PA4M3TQ7VOY3RTMAL4LRRA67ZOHQ65SZD",
    );

    expect(response.status).toBe(200);
    expect(response.body.data.summary.upcomingCount).toBe(1);
    expect(response.body.data.summary.overdueCount).toBe(1);
    expect(response.body.data.upcomingDeadlines[0].grantId).toBe(1);
    expect(response.body.data.overdueMilestones[0].grantId).toBe(1);
  });

  it("returns 404 for a grant that does not exist", async () => {
    const app = createApp(dataSource, sorobanClient);
    const response = await request(app).get("/grants/9999");
    expect(response.status).toBe(404);
  });

  it("returns 400 for a non-numeric grant id", async () => {
    const app = createApp(dataSource, sorobanClient);
    const response = await request(app).get("/grants/not-a-number");
    expect(response.status).toBe(400);
  });

  // -------------------------------------------------------------------------
  // Milestone proofs (unchanged)
  // -------------------------------------------------------------------------

  it("accepts signed milestone proof", async () => {
    const app = createApp(dataSource, sorobanClient);
    const keypair = Keypair.random();
    const timestamp = Date.now();
    const payload = {
      grantId: 1,
      milestoneIdx: 0,
      proofCid: "bafybeihk-example-proof",
      submittedBy: keypair.publicKey(),
      nonce: "nonce-123456",
      timestamp,
    };

    const message = signatureService.buildIntentMessage({ ...payload });
    const signature = keypair
      .sign(Buffer.from(message, "utf8"))
      .toString("base64");

    const response = await request(app).post("/milestone_proof").send({
      ...payload,
      signature,
    });

    expect(response.status).toBe(201);
    expect(response.body.data.grantId).toBe(1);
    expect(response.body.data.milestoneIdx).toBe(0);
  });

  it("rejects milestone proof with invalid signature", async () => {
    const app = createApp(dataSource, sorobanClient);
    const keypair = Keypair.random();

    const response = await request(app).post("/milestone_proof").send({
      grantId: 1,
      milestoneIdx: 1,
      proofCid: "bafybeihk-example-proof-2",
      submittedBy: keypair.publicKey(),
      nonce: "nonce-abcdef12",
      timestamp: Date.now(),
      signature: Buffer.from("x".repeat(64)).toString("base64"),
    });

    expect(response.status).toBe(401);
  });
});
