import { normalizeText } from '../../shared/util.js';

const EMAIL_PATTERN = /^[^\s@<>]+@[^\s@<>]+\.[^\s@<>]+$/u;

export interface ApplicationEmailDraftInput {
  recipientEmail?: unknown;
  roleTitle?: unknown;
  companyName?: unknown;
  candidateName?: unknown;
  candidateEmail?: unknown;
}

export interface ApplicationEmailDraft {
  recipientEmail: string;
  subject: string;
  body: string;
  mailtoUrl: string;
}

function field(value: unknown, name: string, required = false): string {
  const result = normalizeText(value).slice(0, 180);
  if (required && !result) throw new Error(`${name} is required to create the email draft.`);
  return result;
}

export function isValidApplicationEmail(value: unknown): boolean {
  const recipient = normalizeText(value);
  return recipient.length <= 254 && EMAIL_PATTERN.test(recipient);
}

export function createApplicationEmailDraft({
  recipientEmail,
  roleTitle,
  companyName,
  candidateName,
  candidateEmail
}: ApplicationEmailDraftInput): ApplicationEmailDraft {
  const recipient = normalizeText(recipientEmail);
  if (!isValidApplicationEmail(recipient)) {
    throw new Error('A valid employer email address is required.');
  }

  const role = field(roleTitle, 'Role title', true);
  const company = field(companyName, 'Company name', true);
  const name = field(candidateName, 'Candidate name');
  const email = normalizeText(candidateEmail).slice(0, 254);
  const subject = `Application for ${role} at ${company}`;
  const signature = [name, email].filter(Boolean).join('\n');
  const bodyParts = [
    `Hello ${company} hiring team,`,
    '',
    `I am writing to apply for the ${role} role. Please find my resume attached for your review.`,
    '',
    'Regards,'
  ];
  if (signature) bodyParts.push(signature);
  const body = bodyParts.join('\n');
  const mailtoUrl = [
    `mailto:${encodeURIComponent(recipient)}?`,
    `subject=${encodeURIComponent(subject)}`,
    `&body=${encodeURIComponent(body)}`
  ].join('');

  return { recipientEmail: recipient, subject, body, mailtoUrl };
}
