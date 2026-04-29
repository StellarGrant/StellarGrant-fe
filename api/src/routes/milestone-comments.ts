import { Router } from "express";
import { Repository } from "typeorm";
import { z } from "zod";
import { env } from "../config/env";
import { GrantReviewer } from "../entities/GrantReviewer";
import { Milestone } from "../entities/Milestone";
import { MilestoneComment } from "../entities/MilestoneComment";
import { notificationService } from "../services/notification-service";

const createCommentSchema = z.object({
  content: z.string().trim().min(1).max(4000),
  authorAddress: z.string().trim().min(10).max(120),
  parentCommentId: z.number().int().positive().optional(),
});

const isAdmin = (address?: string) => !!address && env.adminAddresses.includes(address);

export const buildMilestoneCommentsRouter = (
  milestoneRepo: Repository<Milestone>,
  commentsRepo: Repository<MilestoneComment>,
  reviewerRepo: Repository<GrantReviewer>,
) => {
  const router = Router();

  router.get("/milestones/:id/comments", async (req, res, next) => {
    try {
      const milestoneId = Number(req.params.id);
      if (Number.isNaN(milestoneId)) {
        res.status(400).json({ error: "Invalid milestone id" });
        return;
      }

      const milestone = await milestoneRepo.findOne({ where: { id: milestoneId } });
      if (!milestone) {
        res.status(404).json({ error: "Milestone not found" });
        return;
      }

      const comments = await commentsRepo.find({
        where: { milestoneId },
        order: { createdAt: "ASC", id: "ASC" },
      });

      res.json({ data: comments });
    } catch (error) {
      next(error);
    }
  });

  router.post("/milestones/:id/comments", async (req, res, next) => {
    try {
      const milestoneId = Number(req.params.id);
      if (Number.isNaN(milestoneId)) {
        res.status(400).json({ error: "Invalid milestone id" });
        return;
      }

      const milestone = await milestoneRepo.findOne({
        where: { id: milestoneId },
        relations: { grant: true },
      });
      if (!milestone) {
        res.status(404).json({ error: "Milestone not found" });
        return;
      }

      const parsed = createCommentSchema.safeParse(req.body);
      if (!parsed.success) {
        res.status(400).json({ error: "Invalid payload", details: parsed.error.issues });
        return;
      }

      const payload = parsed.data;
      if (payload.parentCommentId) {
        const parent = await commentsRepo.findOne({ where: { id: payload.parentCommentId } });
        if (!parent || parent.milestoneId !== milestoneId) {
          res.status(400).json({ error: "Invalid parentCommentId" });
          return;
        }
      }

      const saved = await commentsRepo.save({
        milestoneId,
        content: payload.content,
        authorAddress: payload.authorAddress,
        parentCommentId: payload.parentCommentId ?? null,
      });

      const reviewers = await reviewerRepo.find({ where: { grantId: milestone.grantId } });
      const recipients = new Set<string>([milestone.grant.recipient, ...reviewers.map((reviewer) => reviewer.reviewerStellarAddress)]);
      recipients.delete(payload.authorAddress);

      for (const address of recipients) {
        notificationService.notifyUser(address, "milestone_comment_added", {
          commentId: saved.id,
          milestoneId,
          grantId: milestone.grantId,
          authorAddress: payload.authorAddress,
        });
      }

      res.status(201).json({ data: saved });
    } catch (error) {
      next(error);
    }
  });

  router.delete("/milestones/:id/comments/:commentId", async (req, res, next) => {
    try {
      const milestoneId = Number(req.params.id);
      const commentId = Number(req.params.commentId);
      if (Number.isNaN(milestoneId) || Number.isNaN(commentId)) {
        res.status(400).json({ error: "Invalid id" });
        return;
      }

      const actor = req.header("x-admin-address") ?? undefined;
      if (!isAdmin(actor)) {
        res.status(403).json({ error: "Admin privileges required" });
        return;
      }

      const comment = await commentsRepo.findOne({ where: { id: commentId, milestoneId } });
      if (!comment) {
        res.status(404).json({ error: "Comment not found" });
        return;
      }

      await commentsRepo.delete({ id: comment.id });
      res.json({ ok: true });
    } catch (error) {
      next(error);
    }
  });

  return router;
};
