import { Router } from "express";
import { Repository } from "typeorm";
import { z } from "zod";
import { MilestoneProof } from "../entities/MilestoneProof";
import { SignatureService } from "../services/signature-service";

const milestoneProofSchema = z.object({
  grantId: z.number().int().positive(),
  milestoneIdx: z.number().int().nonnegative(),
  proofCid: z.string().min(3).max(255),
  submittedBy: z.string().min(10).max(120),
  signature: z.string().min(32),
  nonce: z.string().min(8).max(80),
  timestamp: z.number().int().positive(),
});

export const buildMilestoneProofRouter = (
  proofRepo: Repository<MilestoneProof>,
  signatureService: SignatureService,
) => {
  const router = Router();

  router.post("/", async (req, res, next) => {
    try {
      const parsed = milestoneProofSchema.safeParse(req.body);
      if (!parsed.success) {
        res.status(400).json({ error: "Invalid payload", details: parsed.error.issues });
        return;
      }

      const payload = parsed.data;
      const maxSkewMs = 5 * 60 * 1000;
      if (Math.abs(Date.now() - payload.timestamp) > maxSkewMs) {
        res.status(400).json({ error: "Expired intent timestamp" });
        return;
      }

      const signatureIsValid = signatureService.verify(payload);
      if (!signatureIsValid) {
        res.status(401).json({ error: "Invalid Stellar signature" });
        return;
      }

      const proof = await proofRepo.save({
        grantId: payload.grantId,
        milestoneIdx: payload.milestoneIdx,
        proofCid: payload.proofCid,
        submittedBy: payload.submittedBy,
        signature: payload.signature,
        nonce: payload.nonce,
      });

      res.status(201).json({ data: proof });
    } catch (error: any) {
      if (error?.code === "23505" || error?.code === "SQLITE_CONSTRAINT") {
        res.status(409).json({ error: "Proof already submitted for this milestone" });
        return;
      }
      next(error);
    }
  });

  return router;
};
