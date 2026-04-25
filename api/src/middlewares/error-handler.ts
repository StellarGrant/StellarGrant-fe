import { Request, Response, NextFunction } from "express";
import { logger } from "../config/logger";
import { env } from "../config/env";
import { AppError } from "../utils/errors";

export const errorHandler = (
  err: unknown,
  req: Request,
  res: Response,
  _next: NextFunction
): void => {
  const requestId = req.headers["x-request-id"] || "unknown";

  if (err instanceof AppError) {
    logger.error({
      message: err.message,
      errorCode: err.errorCode,
      statusCode: err.statusCode,
      requestId,
      path: req.path,
      method: req.method,
      ...(err.isOperational ? {} : { stack: err.stack }),
    });

    res.status(err.statusCode).json(err.toJSON());
    return;
  }

  if (err instanceof Error) {
    logger.error({
      message: err.message,
      requestId,
      path: req.path,
      method: req.method,
      stack: env.nodeEnv === "development" ? err.stack : undefined,
    });

    const statusCode = (err as any).statusCode || 500;
    res.status(statusCode).json({
      error: env.nodeEnv === "development" ? err.message : "Internal server error",
      errorCode: "INTERNAL_ERROR",
      statusCode,
    });
    return;
  }

  logger.error({
    message: "Unknown error",
    error: err,
    requestId,
    path: req.path,
    method: req.method,
  });

  res.status(500).json({
    error: "Internal server error",
    errorCode: "INTERNAL_ERROR",
    statusCode: 500,
  });
};

export const notFoundHandler = (req: Request, res: Response): void => {
  res.status(404).json({
    error: `Route ${req.method} ${req.path} not found`,
    errorCode: "NOT_FOUND",
    statusCode: 404,
  });
};
