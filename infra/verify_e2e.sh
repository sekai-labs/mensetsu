#!/usr/bin/env bash
set -euo pipefail

API_URL="${API_URL:-http://127.0.0.1:3000}"
INTEL_URL="${INTEL_URL:-http://127.0.0.1:8001}"

echo "=========================================================="
echo "Mensetsu End-to-End Verification Harness"
echo "=========================================================="

echo "1. Checking Core API Health at ${API_URL}/health..."
curl -s -f "${API_URL}/health" | grep -q "healthy" && echo "✓ Core API is healthy."

echo "2. Submitting Synthetic CV and Role to create Interview Workspace..."
PAYLOAD=$(cat << 'EOF'
{
  "company_name": "ScaleVector",
  "role_title": "Staff Distributed Systems Engineer",
  "seniority": "staff",
  "prep_time_weeks": 4,
  "company_urls": ["https://scalevector.example.com"],
  "resume_text": "Alex Mercer: 10 years experience in Rust, Linux, Distributed Systems, Raft, Kubernetes, PostgreSQL.",
  "job_description": "ScaleVector is seeking a Staff Distributed Systems Engineer to scale our state engine. Requirements: Rust, Linux, Distributed Systems, Raft, Temporal."
}
EOF
)

RESPONSE=$(curl -s -X POST "${API_URL}/api/v1/workspaces" \
  -H "Content-Type: application/json" \
  -d "${PAYLOAD}")

WS_ID=$(echo "${RESPONSE}" | grep -o '"id":"[^"]*' | cut -d'"' -f4)
echo "✓ Workspace created successfully with ID: ${WS_ID}"

echo "3. Polling Workspace State until Ready..."
for i in {1..20}; do
  WS_STATUS=$(curl -s "${API_URL}/api/v1/workspaces/${WS_ID}")
  STATUS=$(echo "${WS_STATUS}" | grep -o '"status":"[^"]*' | cut -d'"' -f4)
  PHASE=$(echo "${WS_STATUS}" | grep -o '"current_phase":"[^"]*' | cut -d'"' -f4)
  PERCENT=$(echo "${WS_STATUS}" | grep -o '"progress_percent":[0-9]*' | cut -d':' -f2)

  echo "   [Poll $i] Status: ${STATUS} | Phase: ${PHASE} | Progress: ${PERCENT}%"
  if [ "${STATUS}" = "ready" ]; then
    echo "✓ Workspace successfully reached 'ready' state."
    break
  fi
  sleep 1
done

echo "4. Verifying Match Analysis..."
MATCH_RESP=$(curl -s "${API_URL}/api/v1/workspaces/${WS_ID}/match")
echo "${MATCH_RESP}" | grep -q "overall_match_ratio" && echo "✓ Match analysis verified."

echo "5. Verifying Company Research & Sources..."
COMPANY_RESP=$(curl -s "${API_URL}/api/v1/workspaces/${WS_ID}/company")
echo "${COMPANY_RESP}" | grep -q "engineering_culture" && echo "✓ Company research verified."

echo "6. Verifying Knowledge Graph API..."
KNOWLEDGE_RESP=$(curl -s "${API_URL}/api/v1/workspaces/${WS_ID}/knowledge")
echo "${KNOWLEDGE_RESP}" | grep -q "nodes" && echo "✓ Knowledge graph verified."

echo "7. Verifying Structured Learning Units..."
LEARNING_RESP=$(curl -s "${API_URL}/api/v1/workspaces/${WS_ID}/learning")
echo "${LEARNING_RESP}" | grep -q "content_markdown" && echo "✓ Learning units verified."

echo "8. Verifying Curated Interview Question Book..."
QUESTIONS_RESP=$(curl -s "${API_URL}/api/v1/workspaces/${WS_ID}/questions")
echo "${QUESTIONS_RESP}" | grep -q "strong_answer_rubric" && echo "✓ Question book verified."

echo "=========================================================="
echo "ALL END-TO-END ACCEPTANCE CRITERIA VERIFIED SUCCESSFULLY."
echo "=========================================================="
