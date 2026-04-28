import { Router } from "express";
import { Repository } from "typeorm";
import { User } from "../entities/User";

export const buildUserRouter = (userRepo: Repository<User>) => {
  const router = Router();

  // Register or update user email and notification preferences
  router.post("/register", async (req, res) => {
    const { email, stellarAddress, notifyMilestoneApproved, notifyMilestoneSubmitted } = req.body;
    if (!email || !stellarAddress) {
      return res.status(400).json({ error: "Email and stellarAddress are required" });
    }
    let user = await userRepo.findOne({ where: { stellarAddress } });
    if (user) {
      user.email = email;
      if (notifyMilestoneApproved !== undefined) user.notifyMilestoneApproved = notifyMilestoneApproved;
      if (notifyMilestoneSubmitted !== undefined) user.notifyMilestoneSubmitted = notifyMilestoneSubmitted;
    } else {
      user = userRepo.create({ email, stellarAddress, notifyMilestoneApproved, notifyMilestoneSubmitted });
    }
    await userRepo.save(user);
    res.json({ data: user });
  });

  // Get user notification preferences
  router.get("/:stellarAddress", async (req, res) => {
    const { stellarAddress } = req.params;
    const user = await userRepo.findOne({ where: { stellarAddress } });
    if (!user) return res.status(404).json({ error: "User not found" });
    res.json({ data: user });
  });

  return router;
};
