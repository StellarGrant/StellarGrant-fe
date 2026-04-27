import { Response, NextFunction } from "express";
import { AuthenticatedRequest } from "../types/auth";

// Dummy auth middleware for demonstration. Replace with real auth logic.
export function authMiddleware(req: AuthenticatedRequest, res: Response, next: NextFunction) {
  // In a real app, you would verify a JWT or session and set req.user
  // For now, mock a user for testing
  req.user = {
    stellarAddress: req.headers["x-stellar-address"] as string || "",
    // Add other user fields as needed
  };
  if (!req.user.stellarAddress) {
    return res.status(401).json({ error: "Unauthorized" });
  }
  next();
}
