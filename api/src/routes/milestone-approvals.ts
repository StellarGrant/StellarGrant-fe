import { Router } from "express";
import { Repository } from "typeorm";
import { MilestoneApproval } from "../entities/MilestoneApproval";

export const buildMilestoneApprovalRouter = (approvalRepo: Repository<MilestoneApproval>) => {
  const router = Router();

  // Reviewer approves or rejects a milestone
  router.post("/", async (req, res) => {
    const { grantId, milestoneIdx, reviewerStellarAddress, approved } = req.body;
    if (!grantId || milestoneIdx === undefined || !reviewerStellarAddress || approved === undefined) {
      return res.status(400).json({ error: "grantId, milestoneIdx, reviewerStellarAddress, and approved are required" });
    }
    const approval = approvalRepo.create({ grantId, milestoneIdx, reviewerStellarAddress, approved });
    await approvalRepo.save(approval);
    res.json({ data: approval });
  });

  // List approvals for a milestone
  router.get("/grant/:grantId/milestone/:milestoneIdx", async (req, res) => {
    const { grantId, milestoneIdx } = req.params;
    const approvals = await approvalRepo.find({ where: { grantId: Number(grantId), milestoneIdx: Number(milestoneIdx) } });
    res.json({ data: approvals });
  });

  return router;
};
