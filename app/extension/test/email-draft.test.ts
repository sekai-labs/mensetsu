import { test } from 'node:test';
import assert from 'node:assert/strict';

import { createApplicationEmailDraft, isValidApplicationEmail } from '../src/features/applications/email-draft.js';

test('creates an editable draft from the reviewed role and employer without sending it', () => {
  const draft = createApplicationEmailDraft({
    recipientEmail: 'recruiting+eng@example.com',
    roleTitle: 'Senior Engineer',
    companyName: 'Example Labs',
    candidateName: 'Ari Example',
    candidateEmail: 'ari@example.com'
  });
  const url = new URL(draft.mailtoUrl);

  assert.equal(draft.recipientEmail, 'recruiting+eng@example.com');
  assert.equal(draft.subject, 'Application for Senior Engineer at Example Labs');
  assert.equal(url.protocol, 'mailto:');
  assert.equal(decodeURIComponent(url.pathname), 'recruiting+eng@example.com');
  assert.equal(url.searchParams.get('subject'), draft.subject);
  assert.equal(url.searchParams.get('body'), draft.body);
  assert.match(draft.body, /Senior Engineer/);
  assert.match(draft.body, /Please find my resume attached/);
  assert.match(draft.body, /Ari Example/);
});

test('rejects invalid recipients and incomplete job facts', () => {
  assert.throws(() => createApplicationEmailDraft({ recipientEmail: 'x@example.com\nBcc:evil@example.com', roleTitle: 'Engineer', companyName: 'Example' }), /valid employer email/);
  assert.throws(() => createApplicationEmailDraft({ recipientEmail: 'hiring@example.com', companyName: 'Example' }), /Role title is required/);
  assert.throws(() => createApplicationEmailDraft({ recipientEmail: 'hiring@example.com', roleTitle: 'Engineer' }), /Company name is required/);
});

test('accepts only bounded, syntactically valid employer email addresses', () => {
  assert.equal(isValidApplicationEmail('recruiting+eng@example.com'), true);
  assert.equal(isValidApplicationEmail('x@example.com\nBcc:evil@example.com'), false);
  assert.equal(isValidApplicationEmail(`${'x'.repeat(250)}@x.co`), false);
});
