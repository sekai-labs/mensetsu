import { normalizeText } from '../../shared/util.js';
import type { MatchResult } from '../../shared/types.js';

const MATCH_SEPARATOR = ': ';

function fold(value: unknown): string {
  return normalizeText(value)
    .normalize('NFKD')
    // Vietnamese đ/Đ (U+0111/U+0110) has no canonical decomposition, so map it
    // explicitly before stripping combining marks; otherwise "duoc" never
    // matches "được".
    .replace(/[đĐ]/g, 'd')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase();
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function containsTerm(haystack: unknown, term: unknown): boolean {
  const foldedTerm = fold(term);
  if (!foldedTerm) return false;
  const pattern = new RegExp(
    `(^|[^\\p{L}\\p{N}])${escapeRegExp(foldedTerm)}([^\\p{L}\\p{N}]|$)`,
    'u'
  );
  return pattern.test(fold(haystack));
}

function read(source: unknown, key: string): unknown {
  return source && typeof source === 'object' && !Array.isArray(source)
    ? (source as Record<string, unknown>)[key]
    : undefined;
}

function readText(source: unknown, snake: string, camel: string, fallback = ''): string {
  const value = read(source, snake) ?? read(source, camel) ?? fallback;
  return normalizeText(value);
}

function readList(source: unknown, snake: string, camel: string): string[] {
  const value = read(source, snake) ?? read(source, camel);
  if (!Array.isArray(value)) return [];
  return value.map((item) => normalizeText(item)).filter(Boolean);
}

function criterion(kind: string, value: string): string {
  return `${kind}${MATCH_SEPARATOR}${value}`;
}

interface JobFields {
  roleTitle: string;
  companyName: string;
  jobDescription: string;
  location: string;
}

function jobFields(job: unknown): JobFields {
  const roleTitle = readText(job, 'role_title', 'roleTitle', read(job, 'title') as string ?? '');
  const companyName = readText(job, 'company_name', 'companyName', '');
  const jobDescription = readText(job, 'job_description', 'jobDescription', read(job, 'description') as string ?? '');
  const location = readText(job, 'location', 'location', '');

  return { roleTitle, companyName, jobDescription, location };
}

export function matchDiscoveredJob(job: unknown, profile: unknown = {}): MatchResult {
  if (!job || typeof job !== 'object' || Array.isArray(job)) {
    throw new Error('matchDiscoveredJob requires a discovered job.');
  }

  const { roleTitle, companyName, jobDescription, location } = jobFields(job);
  const corpus = [roleTitle, companyName, location, jobDescription].filter(Boolean).join('\n');

  const targetRole = readText(profile, 'target_role', 'targetRole', '');
  const requiredKeywords = readList(profile, 'required_keywords', 'requiredKeywords');
  const excludedKeywords = readList(profile, 'excluded_keywords', 'excludedKeywords');
  const targetLocations = readList(profile, 'target_locations', 'targetLocations');

  const hasPositivePreferences = Boolean(targetRole)
    || requiredKeywords.length > 0
    || targetLocations.length > 0;

  const matchedCriteria: string[] = [];
  const missingCriteria: string[] = [];
  const excludedMatches: string[] = [];

  if (targetRole) {
    const roleMatches = roleTitle
      && (containsTerm(roleTitle, targetRole) || containsTerm(targetRole, roleTitle));
    if (roleMatches) {
      matchedCriteria.push(criterion('role', targetRole));
    } else {
      missingCriteria.push(criterion('role', targetRole));
    }
  }

  for (const keyword of requiredKeywords) {
    if (containsTerm(corpus, keyword)) {
      matchedCriteria.push(criterion('keyword', keyword));
    } else {
      missingCriteria.push(criterion('keyword', keyword));
    }
  }

  for (const keyword of excludedKeywords) {
    if (containsTerm(corpus, keyword)) excludedMatches.push(keyword);
  }

  if (targetLocations.length > 0) {
    const matchedLocations = targetLocations.filter((target) => (
      location && (containsTerm(location, target) || containsTerm(target, location))
    ));
    if (matchedLocations.length > 0) {
      matchedCriteria.push(...matchedLocations.map((target) => criterion('location', target)));
    } else {
      missingCriteria.push(...targetLocations.map((target) => criterion('location', target)));
    }
  }

  if (!hasPositivePreferences) {
    missingCriteria.push('No targeting preferences configured');
  }

  return {
    eligible: missingCriteria.length === 0 && excludedMatches.length === 0,
    matchedCriteria,
    missingCriteria,
    excludedMatches
  };
}
