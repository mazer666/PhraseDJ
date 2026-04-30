#!/usr/bin/env bash
# check_file_size.sh — Enforce PhraseDJ's 600-line hard limit per source file.
#
# Exit codes:
#   0  All files are within the limit.
#   1  One or more files exceed the limit.
#
# Usage:
#   bash tools/check_file_size.sh           # from repo root
#   bash tools/check_file_size.sh --warn    # warn at 400 lines too

set -euo pipefail

HARD_LIMIT=600
SOFT_LIMIT=400
WARN_ON_SOFT="${1:-}"

EXTENSIONS=("rs" "ts" "tsx" "cpp" "hpp" "cc" "h")
EXCLUDE_DIRS=("target" "node_modules" ".git" "dist" "build" "gen")

violations=0
warnings=0

# Build the find exclusion flags dynamically.
exclude_args=()
for dir in "${EXCLUDE_DIRS[@]}"; do
  exclude_args+=(-not -path "*/$dir/*")
done

# Build the extension filter (find -name "*.rs" -o -name "*.ts" …).
name_args=()
for ext in "${EXTENSIONS[@]}"; do
  if [ ${#name_args[@]} -gt 0 ]; then
    name_args+=(-o)
  fi
  name_args+=(-name "*.${ext}")
done

while IFS= read -r -d '' file; do
  # Count logical code lines (non-blank, non-comment-only lines).
  code_lines=$(grep -cEv '^\s*(//.*)?$' "$file" 2>/dev/null || true)

  if [ "$code_lines" -gt "$HARD_LIMIT" ]; then
    echo "ERROR: $file has $code_lines code lines (limit: $HARD_LIMIT)"
    violations=$((violations + 1))
  elif [ "$WARN_ON_SOFT" = "--warn" ] && [ "$code_lines" -gt "$SOFT_LIMIT" ]; then
    echo "WARN:  $file has $code_lines code lines (soft limit: $SOFT_LIMIT)"
    warnings=$((warnings + 1))
  fi
done < <(find . "${exclude_args[@]}" \( "${name_args[@]}" \) -print0)

if [ "$violations" -gt 0 ]; then
  echo ""
  echo "$violations file(s) exceed the $HARD_LIMIT-line hard limit."
  echo "Split them into smaller modules (see LLM.md Rule 1)."
  exit 1
fi

if [ "$warnings" -gt 0 ]; then
  echo ""
  echo "$warnings file(s) are above the $SOFT_LIMIT-line soft limit (informational)."
fi

echo "File-length check passed."
