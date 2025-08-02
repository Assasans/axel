#!/bin/bash

# Copyright (c) 2025 Daniil Pryima <swimmin2@gmail.com>. Licensed under the MIT License.
# See LICENSE-MIT in the repository root for full license text.
#
# SPDX-FileCopyrightText: 2025 Daniil Pryima <swimmin2@gmail.com>
# SPDX-License-Identifier: MIT

set -e

FRESH_MODE=false
HELP=false
FRESH_VERSION=""
CONNECTION_ARGS=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --help|-h)
      HELP=true
      shift
      ;;
    --fresh)
      FRESH_MODE=true
      # Check if next argument is a version number (not a flag or connection string)
      if [ -n "$2" ] && [[ $2 =~ ^[0-9]+$ ]]; then
        FRESH_VERSION="$2"
        shift 2
      else
        shift
      fi
      ;;
    --fresh=*)
      FRESH_MODE=true
      FRESH_VERSION="${1#--fresh=}"
      # Check that FRESH_VERSION is a valid number
      if [[ ! $FRESH_VERSION =~ ^[0-9]+$ ]]; then
        echo "Error: Invalid version number for --fresh option"
        exit 1
      fi
      shift
      ;;
    *)
      if [ -z "$CONNECTION_ARGS" ]; then
        CONNECTION_ARGS="$1"
      else
        echo "Error: Multiple connection strings provided"
        exit 1
      fi
      shift
      ;;
  esac
done

if [ "$HELP" = true ] || [ -z "$CONNECTION_ARGS" ]; then
  echo "Usage: $0 [--fresh [version]] <connection commands>"
  echo ""
  echo "Options:"
  echo "      --fresh            apply all migrations from scratch, ignoring applied migrations table"
  echo "      --fresh <version>  apply migrations from the specified version onwards"
  echo "  -h, --help             display this help and exit"
  echo ""
  echo "Example: $0 \"host=localhost dbname=myapp user=postgres\""
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for better readability
GREEN=$(tput setaf 2)
YELLOW=$(tput setaf 3)
RED=$(tput setaf 1)
GRAY=$(tput setaf 8)
NC=$(tput sgr0)
BOLD=$(tput bold)
INVERTED=$(tput smso)

# Ensure the migrations table exists
# echo -e "Ensuring migrations table exists...${NC}"
PGOPTIONS="-c client_min_messages=error" psql "$CONNECTION_ARGS" -q -c "
CREATE TABLE IF NOT EXISTS schema_migrations (
  version VARCHAR(255) PRIMARY KEY,
  applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  description TEXT
);"

# Get list of applied migrations (unless in fresh mode)
if [ "$FRESH_MODE" = true ]; then
  if [ -n "$FRESH_VERSION" ]; then
    true
    # echo -e "${YELLOW}${BOLD}Fresh mode from version $FRESH_VERSION - ignoring applied migrations${NC}"
  else
    true
    # echo -e "${YELLOW}${BOLD}Fresh mode enabled - ignoring applied migrations${NC}"
  fi
  APPLIED_MIGRATIONS=""
else
  echo -e "${GRAY}Fetching applied migrations...${NC}"
  APPLIED_MIGRATIONS=$(psql "$CONNECTION_ARGS" -t -c "SELECT version FROM schema_migrations ORDER BY version;" | tr -d ' ')
fi

# Find all migration files in the script directory
MIGRATION_FILES=($(find "$SCRIPT_DIR" -maxdepth 1 -name "*.sql" | sort))

# Display current migration status
# if [ "$FRESH_MODE" = true ]; then
#   if [ -n "$FRESH_VERSION" ]; then
#     echo -e "${YELLOW}Fresh mode from version $FRESH_VERSION${NC}"
#   else
#     echo -e "${YELLOW}Fresh mode: all migrations will be applied${NC}"
#   fi
# fi
echo -e "${BOLD}Current migration status:${NC}"
echo "─────────────────────────────────────────────────────────────────────"
printf "${BOLD}%-20s %-40s %s\n" "VERSION" "DESCRIPTION" "STATUS ${NC}"

PENDING_MIGRATIONS=()
APPLIED_COUNT=0
PENDING_COUNT=0
SKIPPED_COUNT=0

for MIGRATION_FILE in "${MIGRATION_FILES[@]}"; do
  FILENAME=$(basename "$MIGRATION_FILE")
  # Extract version and description
  if [[ $FILENAME =~ ^([0-9]+)_(.+)\.sql$ ]]; then
    VERSION="${BASH_REMATCH[1]}"
    DESCRIPTION="${BASH_REMATCH[2]//[_]/ }"

    # Skip migrations before the specified fresh version
    if [ -n "$FRESH_VERSION" ] && [ "$VERSION" -lt "$FRESH_VERSION" ]; then
      printf "%-20s %-40s %s\n" "$VERSION" "$DESCRIPTION" "${GRAY}SKIP   ${NC}"
      SKIPPED_COUNT=$((SKIPPED_COUNT + 1))
      continue
    fi

    # Check if migration was already applied (unless in fresh mode)
    if [ "$FRESH_MODE" = true ]; then
      printf "%-20s %-40s %s\n" "$VERSION" "$DESCRIPTION" "${RED}${INVERTED}PENDING${NC}"
      PENDING_MIGRATIONS+=("$MIGRATION_FILE")
      PENDING_COUNT=$((PENDING_COUNT + 1))
    elif echo "$APPLIED_MIGRATIONS" | grep -q "$VERSION"; then
      printf "%-20s %-40s %s\n" "$VERSION" "$DESCRIPTION" "${GREEN}APPLIED${NC}"
      APPLIED_COUNT=$((APPLIED_COUNT + 1))
    else
      printf "%-20s %-40s %s\n" "$VERSION" "$DESCRIPTION" "${YELLOW}${INVERTED}PENDING${NC}"
      PENDING_MIGRATIONS+=("$MIGRATION_FILE")
      PENDING_COUNT=$((PENDING_COUNT + 1))
    fi
  else
    echo -e "${RED}Warning: File '$FILENAME' does not match migration naming pattern (should be VERSION_description.sql)${NC}"
  fi
done

echo "─────────────────────────────────────────────────────────────────────"
if [ "$FRESH_MODE" = true ]; then
  if [ -n "$FRESH_VERSION" ]; then
    echo -e "${BOLD}$SKIPPED_COUNT migration$( [ "$SKIPPED_COUNT" -eq 1 ] || echo 's' ) skipped, ${YELLOW}${PENDING_COUNT} pending${NC}${BOLD}:"
  else
    echo -e "${YELLOW}${BOLD}$PENDING_COUNT migrations pending:${NC}"
  fi
else
  echo -e "${BOLD}$APPLIED_COUNT migration$( [ "$APPLIED_COUNT" -eq 1 ] || echo 's' ) applied, ${YELLOW}$PENDING_COUNT pending${NC}${BOLD}:"
fi
for MIGRATION in "${PENDING_MIGRATIONS[@]}"; do
  FILENAME=$(basename "$MIGRATION")
  echo "${YELLOW}  - ${BOLD}$FILENAME${NC}"
done

# Exit if no pending migrations
if [ ${#PENDING_MIGRATIONS[@]} -eq 0 ]; then
  echo -e "\n${GREEN}Database is up to date."
  exit 0
fi

# Ask for confirmation
if [ "$FRESH_MODE" = true ]; then
  if [ -n "$FRESH_VERSION" ]; then
    echo -e "\n${RED}${BOLD}FRESH MODE: Migrations starting from version $FRESH_VERSION will be applied, DATA LOSS IS POSSIBLE:${NC}"
  else
    echo -e "\n${RED}${BOLD}FRESH MODE: All migrations will be applied, ALL DATA WILL BE LOST!${NC}"
  fi
fi

echo
read -p "Do you want to proceed with these migrations? [y/N] " -n 1 -r

echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo -e "${RED}✗ Migration aborted by user.${NC}"
  exit 0
fi

MIGRATION_WRAPPER_FILE="/tmp/migration_wrapper_$(date +%s).sql"

# Apply pending migrations
# echo -e "\n${YELLOW}Applying migrations...${NC}"
for MIGRATION in "${PENDING_MIGRATIONS[@]}"; do
  FILENAME=$(basename "$MIGRATION")
  if [[ $FILENAME =~ ^([0-9]+)_(.+)\.sql$ ]]; then
    VERSION="${BASH_REMATCH[1]}"
    DESCRIPTION="${BASH_REMATCH[2]//[_]/ }"

    echo -e "${YELLOW}⏳ Applying migration:${NC} ${BOLD}$FILENAME${NC}"

    # Run migration inside a transaction
    echo "BEGIN;" > "$MIGRATION_WRAPPER_FILE"
    cat "$MIGRATION" >> "$MIGRATION_WRAPPER_FILE"
    # Record successful migration (use UPSERT for fresh mode)
    if [ "$FRESH_MODE" = true ]; then
      echo "INSERT INTO schema_migrations (version, description) VALUES ('$VERSION', '$DESCRIPTION') ON CONFLICT (version) DO UPDATE SET applied_at = CURRENT_TIMESTAMP, description = EXCLUDED.description;" >> "$MIGRATION_WRAPPER_FILE"
    else
      echo "INSERT INTO schema_migrations (version, description) VALUES ('$VERSION', '$DESCRIPTION');" >> "$MIGRATION_WRAPPER_FILE"
    fi
    echo "COMMIT;" >> "$MIGRATION_WRAPPER_FILE"

    if psql "$CONNECTION_ARGS" -v ON_ERROR_STOP=1 -f "$MIGRATION_WRAPPER_FILE"; then
      echo -e "${GREEN}✓ Migration ${BOLD}$FILENAME${NC}${GREEN} applied successfully.${NC}"
    else
      echo -e "${RED}✗ Migration ${BOLD}$FILENAME${NC}${RED} failed.${NC}"
      exit 1
    fi
  fi
done

rm -f "$MIGRATION_WRAPPER_FILE"
echo -e "\n${GREEN}✓ Applied $PENDING_COUNT migrations.${NC}"
