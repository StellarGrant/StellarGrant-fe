import nodemailer, { Transporter } from "nodemailer";
import { env } from "../config/env";
import { logger } from "../config/logger";

export type EmailEventType =
  | "milestone_submitted"
  | "milestone_approved"
  | "milestone_rejected"
  | "grant_funded"
  | "grant_created";

export interface EmailPayload {
  to: string;
  event: EmailEventType;
  data: Record<string, string | number>;
}

interface EmailTemplate {
  subject: string;
  html: string;
  text: string;
}

function buildTemplate(event: EmailEventType, data: Record<string, string | number>): EmailTemplate {
  const base = env.appBaseUrl;
  switch (event) {
    case "milestone_submitted":
      return {
        subject: `[StellarGrant] Milestone #${data.milestoneIdx} submitted for review — Grant #${data.grantId}`,
        html: `<p>A new milestone proof has been submitted for <strong>Grant #${data.grantId}</strong> (Milestone ${data.milestoneIdx}).</p>
               <p>Submitted by: <code>${data.submittedBy}</code></p>
               <p><a href="${base}/grants/${data.grantId}/milestones/${data.milestoneIdx}">Review milestone</a></p>`,
        text: `Milestone #${data.milestoneIdx} submitted for Grant #${data.grantId} by ${data.submittedBy}.\nReview at: ${base}/grants/${data.grantId}/milestones/${data.milestoneIdx}`,
      };
    case "milestone_approved":
      return {
        subject: `[StellarGrant] Milestone #${data.milestoneIdx} approved — Grant #${data.grantId}`,
        html: `<p>Your milestone proof for <strong>Grant #${data.grantId}</strong> (Milestone ${data.milestoneIdx}) has been <strong>approved</strong>.</p>
               <p><a href="${base}/grants/${data.grantId}">View grant</a></p>`,
        text: `Your milestone #${data.milestoneIdx} for Grant #${data.grantId} was approved.\nView at: ${base}/grants/${data.grantId}`,
      };
    case "milestone_rejected":
      return {
        subject: `[StellarGrant] Milestone #${data.milestoneIdx} rejected — Grant #${data.grantId}`,
        html: `<p>Your milestone proof for <strong>Grant #${data.grantId}</strong> (Milestone ${data.milestoneIdx}) has been <strong>rejected</strong>.</p>
               ${data.reason ? `<p>Reason: ${data.reason}</p>` : ""}
               <p><a href="${base}/grants/${data.grantId}/milestones/${data.milestoneIdx}">Resubmit</a></p>`,
        text: `Your milestone #${data.milestoneIdx} for Grant #${data.grantId} was rejected.${data.reason ? ` Reason: ${data.reason}` : ""}\nResubmit at: ${base}/grants/${data.grantId}/milestones/${data.milestoneIdx}`,
      };
    case "grant_funded":
      return {
        subject: `[StellarGrant] Funding received for Grant #${data.grantId}`,
        html: `<p>Your grant <strong>${data.grantTitle ?? `#${data.grantId}`}</strong> has received funding of <strong>${data.amount}</strong>.</p>
               <p><a href="${base}/grants/${data.grantId}">View grant</a></p>`,
        text: `Grant #${data.grantId} received funding of ${data.amount}.\nView at: ${base}/grants/${data.grantId}`,
      };
    case "grant_created":
      return {
        subject: `[StellarGrant] New grant created — #${data.grantId}`,
        html: `<p>A new grant has been created: <strong>${data.grantTitle ?? `#${data.grantId}`}</strong></p>
               <p>Creator: <code>${data.recipient}</code></p>
               <p><a href="${base}/grants/${data.grantId}">View grant</a></p>`,
        text: `New grant #${data.grantId} created by ${data.recipient}.\nView at: ${base}/grants/${data.grantId}`,
      };
  }
}

export class EmailService {
  private transporter: Transporter;

  constructor() {
    this.transporter = nodemailer.createTransport({
      host: env.smtpHost,
      port: env.smtpPort,
      secure: env.smtpPort === 465,
      auth: env.smtpUser
        ? { user: env.smtpUser, pass: env.smtpPass }
        : undefined,
    });
  }

  async send(payload: EmailPayload): Promise<void> {
    const template = buildTemplate(payload.event, payload.data);
    try {
      await this.transporter.sendMail({
        from: env.emailFrom,
        to: payload.to,
        subject: template.subject,
        html: template.html,
        text: template.text,
      });
      logger.info("Email sent", { event: payload.event, to: payload.to });
    } catch (err) {
      logger.error("Failed to send email", { event: payload.event, to: payload.to, err });
      throw err;
    }
  }
}

export const emailService = new EmailService();
