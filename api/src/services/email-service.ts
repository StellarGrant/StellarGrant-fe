import sgMail from '@sendgrid/mail';

export type EmailEventType =
  | "milestone_submitted"
  | "milestone_approved"
  | "milestone_rejected"
  | "grant_funded"
  | "grant_created"
  | "milestone_deadline_upcoming"
  | "milestone_deadline_overdue";

export type EmailType = 'milestone_approved' | 'milestone_submitted';

interface SendEmailParams {
  to: string;
  subject: string;
  html: string;
}

export async function sendEmail({ to, subject, html }: SendEmailParams) {
  const msg = {
    to,
    from: process.env.SENDGRID_FROM_EMAIL!,
    subject,
    html,
  };
  await sgMail.send(msg);
}

export function getEmailTemplate(type: EmailType, data: Record<string, any>): { subject: string; html: string } {
  switch (type) {
    case 'milestone_approved':
      return {
        subject: `Milestone Approved: ${data.grantTitle}`,
        html: `<p>Congratulations! Your milestone <b>${data.milestoneTitle}</b> for grant <b>${data.grantTitle}</b> has been approved.</p>`
      };
    case 'milestone_submitted':
      return {
        subject: `New Milestone Submission: ${data.grantTitle}`,
        html: `<p>A new milestone <b>${data.milestoneTitle}</b> has been submitted for grant <b>${data.grantTitle}</b>. Please review it.</p>`
      };
    case "milestone_deadline_upcoming":
      return {
        subject: `[StellarGrant] Milestone due in ${data.daysRemaining} day${Number(data.daysRemaining) === 1 ? "" : "s"} — Grant #${data.grantId}`,
        html: `<p>Your milestone <strong>${data.milestoneTitle ?? `#${data.milestoneIdx}`}</strong> for <strong>${data.grantTitle ?? `Grant #${data.grantId}`}</strong> is due in <strong>${data.daysRemaining} day${Number(data.daysRemaining) === 1 ? "" : "s"}</strong>.</p>
               <p>Deadline: <code>${data.deadline}</code></p>
               <p><a href="${base}/grants/${data.grantId}/milestones">Open milestone dashboard</a></p>`,
        text: `Milestone ${data.milestoneTitle ?? `#${data.milestoneIdx}`} for Grant #${data.grantId} is due in ${data.daysRemaining} day${Number(data.daysRemaining) === 1 ? "" : "s"}.\nDeadline: ${data.deadline}\nOpen: ${base}/grants/${data.grantId}/milestones`,
      };
    case "milestone_deadline_overdue":
      return {
        subject: `[StellarGrant] Milestone overdue — Grant #${data.grantId}`,
        html: `<p>Your milestone <strong>${data.milestoneTitle ?? `#${data.milestoneIdx}`}</strong> for <strong>${data.grantTitle ?? `Grant #${data.grantId}`}</strong> is now <strong>overdue</strong>.</p>
               <p>Deadline: <code>${data.deadline}</code></p>
               <p><a href="${base}/grants/${data.grantId}/milestones">Review milestone</a></p>`,
        text: `Milestone ${data.milestoneTitle ?? `#${data.milestoneIdx}`} for Grant #${data.grantId} is overdue.\nDeadline: ${data.deadline}\nReview: ${base}/grants/${data.grantId}/milestones`,
      };
  }
}
