import { Router } from "express";
import { Repository } from "typeorm";
import { z } from "zod";
import { MilestoneProof } from "../entities/MilestoneProof";
import { Activity } from "../entities/Activity";
import { SignatureService } from "../services/signature-service";
import { Grant } from "../entities/Grant";
import { User } from "../entities/User";
import * as emailService from "../services/email-service";
import { notificationService } from "../services/notification-service";

const milestoneProofSchema = z.object({
  grantId: z.number().int().positive(),
  milestoneIdx: z.number().int().nonnegative(),
  proofCid: z.string().min(3).max(255),
  description: z.string().optional(),
  submittedBy: z.string().min(10).max(120),
  signature: z.string().min(32),
  nonce: z.string().min(8).max(80),
  timestamp: z.number().int().positive(),
});

export const buildMilestoneProofRouter = (
  proofRepo: Repository<MilestoneProof>,
  signatureService: SignatureService,
  grantRepo?: Repository<Grant>,
  userRepo?: Repository<User>,
) => {
  const activityRepo = proofRepo.manager.getRepository(Activity);
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
        description: payload.description || null,
        submittedBy: payload.submittedBy,
        signature: payload.signature,
        nonce: payload.nonce,
      });

      // Email notification logic
      if (grantRepo && userRepo) {
        // Load reviewers relation so we can notify reviewers as well
        const grant = await grantRepo.findOne({ where: { id: payload.grantId }, relations: ["reviewers"] });
        if (grant) {
          const owner = await userRepo.findOne({ where: { stellarAddress: grant.recipient } });
          if (owner && owner.email && owner.notifyMilestoneSubmitted) {
            const emailData = {
              grantTitle: grant.title,
              milestoneTitle: `#${payload.milestoneIdx}`,
            };
            const { subject, html } = emailService.getEmailTemplate('milestone_submitted', emailData);
            await emailService.sendEmail({ to: owner.email, subject, html });
          }

          // Notify reviewers who opted in
          if ((grant as any).reviewers && Array.isArray((grant as any).reviewers)) {
            for (const grReviewer of (grant as any).reviewers) {
              if (!grReviewer) continue;
              // Always use reviewerStellarAddress for user lookup
              if (!grReviewer.reviewerStellarAddress) continue;
              const reviewerUser = await userRepo.findOne({ where: { stellarAddress: grReviewer.reviewerStellarAddress } });
              if (reviewerUser && reviewerUser.email && reviewerUser.notifyMilestoneSubmitted) {
                const emailData = {
                  grantTitle: grant.title,
                  milestoneTitle: `#${payload.milestoneIdx}`,
                };
                const { subject, html } = emailService.getEmailTemplate('milestone_submitted', emailData);
                await emailService.sendEmail({ to: reviewerUser.email, subject, html });
              }
            }
          }
        }
      }

      await responseCache.invalidateGrantsAndStats();
      // Broadcast to reviewers (simplified for now as broadcast)
      notificationService.broadcast("milestone_submitted", {
        grantId: payload.grantId,
        milestoneIdx: payload.milestoneIdx,
        submittedBy: payload.submittedBy
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
