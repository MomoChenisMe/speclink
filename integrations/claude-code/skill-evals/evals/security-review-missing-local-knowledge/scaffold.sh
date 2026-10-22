#!/usr/bin/env bash
source "$(dirname "$0")/../fixtures.sh"
eval_greeting_repo
eval_greeting_change
cat >>openspec/config.yaml <<'YAML'
security_verification:
  enabled: true
  required_for_acceptance: true
  standards:
    owasp_top10: {enabled: true, version: '2021'}
  reviews: {spec: true, code: true, api_boundary: false}
YAML
eval_commit 'selected security policy without installed knowledge'
