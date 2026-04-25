import winston from "winston";
import { env } from "./env";

const logLevel = env.logLevel;

const isProduction = env.nodeEnv === "production";

const logFormat = winston.format.combine(
  winston.format.timestamp({ format: "isoDateTime" }),
  winston.format.errors({ stack: true }),
  isProduction
    ? winston.format.json()
    : winston.format.combine(
        winston.format.colorize(),
        winston.format.printf(({ level, message, timestamp, stack, ...metadata }) => {
          let msg = `${timestamp} [${level}]: ${message}`;
          if (stack) {
            msg += `\n${stack}`;
          }
          if (Object.keys(metadata).length > 0) {
            msg += `\n${JSON.stringify(metadata, null, 2)}`;
          }
          return msg;
        })
      )
);

export const logger = winston.createLogger({
  level: logLevel,
  format: logFormat,
  transports: [
    new winston.transports.Console({
      silent: process.env.NODE_ENV === "test",
    }),
  ],
  exceptionHandlers: [
    new winston.transports.Console({
      silent: process.env.NODE_ENV === "test",
    }),
  ],
  rejectionHandlers: [
    new winston.transports.Console({
      silent: process.env.NODE_ENV === "test",
    }),
  ],
});

export const requestLogger = (): winston.Logger =>
  winston.createLogger({
    level: logLevel,
    format: isProduction
      ? winston.format.combine(
          winston.format.timestamp({ format: "isoDateTime" }),
          winston.format.json()
        )
      : winston.format.combine(
          winston.format.colorize(),
          winston.format.printf(({ level, message, timestamp, ...metadata }) => {
            return `${timestamp} [${level}]: ${message} ${JSON.stringify(metadata)}`;
          })
        ),
    transports: [
      new winston.transports.Console({
        silent: process.env.NODE_ENV === "test",
      }),
    ],
  });
