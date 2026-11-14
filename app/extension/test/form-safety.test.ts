import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { parseHtml } from './fixtures/dom-helper.js';
import { fillPreparedFields } from '../src/content/adapters/dom.js';
import { greenhouseAdapter } from '../src/content/adapters/greenhouse.js';
import type { PreparedFields } from '../src/shared/types.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const evidenceDir = path.resolve(__dirname, '..', 'evidence');

describe('Form Safety and Guardrails', () => {
  const greenhouseDir = path.join(evidenceDir, 'greenhouse-2026-11-10');
  const formHtml = fs.readFileSync(path.join(greenhouseDir, 'job-post.application-form.sanitized.html'), 'utf8');

  it('fills only safe named fields and attaches resume on Greenhouse evidence', () => {
    const doc = parseHtml(formHtml, 'Greenhouse Job');
    const preparedFields: PreparedFields = {
      first_name: 'Jane',
      last_name: 'Doe',
      email: 'jane.doe@example.com',
      phone: '+1-555-0199',
      location: 'Bangalore, India'
    };

    const dummyPdf = new Uint8Array([0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x37]);
    const resume = { bytes: dummyPdf, filename: 'candidate_resume.pdf', mime: 'application/pdf' };
    const previousFile = globalThis.File;
    const previousDataTransfer = globalThis.DataTransfer;
    globalThis.File = class {
      name: string;
      type: string;
      size: number;
      constructor(parts: Array<{ byteLength: number }>, name: string, options: { type?: string } = {}) {
        this.name = name;
        this.type = options.type || '';
        this.size = parts.reduce((total, part) => total + part.byteLength, 0);
      }
    } as unknown as typeof File;
    globalThis.DataTransfer = class {
      files: Array<{ name: string; type: string; size: number }>;
      items: { add: (file: { name: string; type: string; size: number }) => void };
      constructor() {
        this.files = [];
        this.items = { add: (file: { name: string; type: string; size: number }) => this.files.push(file) };
      }
    } as unknown as typeof DataTransfer;
    let result;
    try {
      result = greenhouseAdapter.fill(doc, preparedFields, resume);
    } finally {
      if (previousFile === undefined) delete (globalThis as { File?: unknown }).File;
      else globalThis.File = previousFile;
      if (previousDataTransfer === undefined) delete (globalThis as { DataTransfer?: unknown }).DataTransfer;
      else globalThis.DataTransfer = previousDataTransfer;
    }

    const form = doc.querySelector('form#application-form');
    assert.ok(form, 'Form was found');
    assert.strictEqual(form.querySelector('#first_name')?.value, 'Jane');
    assert.strictEqual(form.querySelector('#last_name')?.value, 'Doe');
    assert.strictEqual(form.querySelector('#email')?.value, 'jane.doe@example.com');
    assert.strictEqual(form.querySelector('#phone')?.value, '+1-555-0199');

    assert.strictEqual(result.resumeAttached, true);
    assert.strictEqual(form.querySelector('#resume')?.files?.[0]?.name, 'candidate_resume.pdf');
    assert.strictEqual(form.querySelector('#resume')?.files?.[0]?.type, 'application/pdf');
  });

  it('reports the selected resume as unattached whenever the attachment step fails', () => {
    const doc = parseHtml(formHtml, 'Greenhouse Job');
    const form = doc.querySelector('form#application-form');
    assert.ok(form, 'Form was found');
    const input = form.querySelector('#resume');
    assert.ok(input, 'Resume input was found');
    input.files = [{ name: 'pre-existing.pdf', size: 12, type: 'application/pdf' }];

    const resume = {
      bytes: new Uint8Array([0x25, 0x50, 0x44, 0x46]),
      filename: 'candidate_resume.pdf',
      mime: 'application/pdf'
    };
    const result = greenhouseAdapter.fill(doc, { first_name: 'Jane' }, resume);

    assert.strictEqual(result.resumeAttached, false);
    assert.strictEqual(result.resume?.attached, false);
    assert.strictEqual(result.resume?.reason, 'file_api_unavailable');
  });

  it('never fills passwords, checkboxes, or sensitive EEO demographic fields', () => {
    const adversarialHtml = `
      <form id="test-form">
        <input type="text" id="first_name" name="first_name" />
        <input type="email" id="email" name="email" />
        <input type="password" id="password" name="password" />
        <input type="checkbox" id="terms_agree" name="terms_agree" />
        <input type="checkbox" id="consent_checkbox" name="consent_checkbox" />
        <input type="text" id="gender" name="gender" />
        <input type="text" id="race_ethnicity" name="race_ethnicity" />
        <input type="text" id="veteran_status" name="veteran_status" />
        <input type="text" id="disability" name="disability" />
        <button type="submit">Submit</button>
      </form>
    `;

    const doc = parseHtml(adversarialHtml);
    const form = doc.querySelector('#test-form');
    assert.ok(form, 'Form was found');

    const preparedFields: PreparedFields = {
      first_name: 'Alex',
      email: 'alex@example.com',
      password: 'hacked_password123',
      terms_agree: 'true',
      consent_checkbox: true,
      gender: 'Non-binary',
      race_ethnicity: 'Decline to self-identify',
      veteran_status: 'Not a veteran',
      disability: 'No disability'
    };

    const report = fillPreparedFields(form, preparedFields);

    assert.strictEqual(form.querySelector('#first_name')?.value, 'Alex');
    assert.strictEqual(form.querySelector('#email')?.value, 'alex@example.com');
    assert.strictEqual(form.querySelector('#password')?.value, '');
    assert.strictEqual(form.querySelector('#terms_agree')?.checked, false);
    assert.strictEqual(form.querySelector('#consent_checkbox')?.checked, false);
    assert.strictEqual(form.querySelector('#gender')?.value, '');
    assert.strictEqual(form.querySelector('#race_ethnicity')?.value, '');
    assert.strictEqual(form.querySelector('#veteran_status')?.value, '');
    assert.strictEqual(form.querySelector('#disability')?.value, '');

    const skippedKeys = report.skipped.map((entry) => entry.key);
    assert.ok(skippedKeys.includes('password'));
    assert.ok(skippedKeys.includes('gender'));
    assert.ok(skippedKeys.includes('race_ethnicity'));
  });

  it('never submits while filling', () => {
    const doc = parseHtml(formHtml, 'Greenhouse Job');
    const form = doc.querySelector('form#application-form');
    assert.ok(form, 'Form was found');
    const submitBtn = form.querySelector('button[type="submit"]');
    assert.ok(submitBtn, 'Submit control was found');

    const listenerEvents: Array<{ type: string }> = [];
    form.addEventListener('submit', (event) => {
      listenerEvents.push(event);
    });

    const preparedFields: PreparedFields = {
      first_name: 'Safe',
      last_name: 'Candidate',
      email: 'safe@example.com'
    };

    greenhouseAdapter.fill(doc, preparedFields, null);

    assert.strictEqual(listenerEvents.length, 0, 'Submit event must not be fired during fill');
    assert.strictEqual(submitBtn.clicked, false, 'Submit button must not be clicked during fill');
  });
});
