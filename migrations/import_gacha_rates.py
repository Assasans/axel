#!/usr/bin/env python3
"""
Import gacha rates from JSON files into PostgreSQL database.

Usage: python import_gacha_rates.py <directory_path> <postgresql_connection_string>
Example: python import_gacha_rates.py ./gacha_rates "postgresql://user:password@localhost:5432/dbname"
"""

import sys
import json
from pathlib import Path
import psycopg2
from psycopg2.extras import execute_batch


def process_json_file(filepath):
  """Parse a JSON file and return structured data for insertion."""
  with open(filepath, 'r', encoding='utf-8') as f:
    data = json.load(f)

  gacha_id = data['gacha_id']

  # Build a dictionary of limitrate items for quick lookup
  limitrate_dict = {}
  for item in data.get('limitrate', []):
    limitrate_dict[item['itemid']] = item['rate']

  # Process gacha_rates data
  gacha_rates = []
  for item in data.get('rate', []):
    item_id = item['itemid']
    probability = item['rate'] / 1000.0  # JSON rates are multiplied by 1000

    # Get probability_pity from limitrate if exists
    probability_pity = None
    if item_id in limitrate_dict:
      probability_pity = limitrate_dict[item_id] / 1000.0

    is_rate_up = bool(item.get('pickup', 0))

    # details_priority should be null if detailview is 0, otherwise set to detailpriority
    detailview = item.get('detailview', 0)
    details_priority = None if detailview == 0 else item.get('detailpriority', 0)

    gacha_rates.append({
      'gacha_id': gacha_id,
      'item_id': item_id,
      'probability': probability,
      'probability_pity': probability_pity,
      'is_rate_up': is_rate_up,
      'details_priority': details_priority
    })

  # Process gacha_bonus_rates data
  gacha_bonus_rates = []
  for bonus in data.get('bonusrate', []):
    pack_id = bonus['pack_id']
    probability = bonus['rate'] / 1000.0  # JSON rates are multiplied by 1000

    gacha_bonus_rates.append({
      'gacha_id': gacha_id,
      'pack_id': pack_id,
      'probability': probability
    })

  return gacha_rates, gacha_bonus_rates


def insert_data(conn, all_gacha_rates, all_gacha_bonus_rates):
  """Insert data into PostgreSQL using batched inserts."""
  with conn.cursor() as cur:
    # Insert gacha_rates
    if all_gacha_rates:
      insert_gacha_rates_sql = """
                               INSERT INTO gacha.rates (gacha_id, item_id, probability, probability_pity, is_rate_up,
                                                        details_priority)
                               VALUES (%(gacha_id)s, %(item_id)s, %(probability)s, %(probability_pity)s, %(is_rate_up)s,
                                       %(details_priority)s) \
                               """
      execute_batch(cur, insert_gacha_rates_sql, all_gacha_rates, page_size=1000)
      print(f"✓ Inserted/updated {len(all_gacha_rates)} gacha_rates records")

    # Insert gacha_bonus_rates
    if all_gacha_bonus_rates:
      insert_gacha_bonus_rates_sql = """
                                     INSERT INTO gacha.bonus_rates (gacha_id, pack_id, probability)
                                     VALUES (%(gacha_id)s, %(pack_id)s,
                                             %(probability)s) \
                                     """
      execute_batch(cur, insert_gacha_bonus_rates_sql, all_gacha_bonus_rates, page_size=1000)
      print(f"✓ Inserted/updated {len(all_gacha_bonus_rates)} gacha_bonus_rates records")

    conn.commit()


def main():
  if len(sys.argv) < 3:
    print("Usage: python import_gacha_rates.py <postgresql_connection_string> <directory_path>")
    print("Example: python import_gacha_rates.py 'postgresql://user:password@localhost:5432/dbname' ./data")
    sys.exit(1)

  directory_path = sys.argv[1]
  connection_string = sys.argv[2]

  # Connect to PostgreSQL
  try:
    conn = psycopg2.connect(connection_string)
    print(f"✓ Connected to database successfully")
  except Exception as e:
    print(f"✗ Failed to connect to database: {e}")
    sys.exit(1)

  try:
    # Collect all JSON files
    directory = Path(directory_path)
    if not directory.exists():
      print(f"✗ Directory not found: {directory_path}")
      sys.exit(1)

    json_files = list(directory.glob('*.json'))

    if not json_files:
      print(f"✗ No JSON files found in {directory_path}")
      return

    print(f"✓ Found {len(json_files)} JSON files\n")

    all_gacha_rates = []
    all_gacha_bonus_rates = []

    # Process all JSON files
    for i, json_file in enumerate(json_files, 1):
      print(f"[{i}/{len(json_files)}] Processing {json_file.name}.. .", end=' ')
      try:
        gacha_rates, gacha_bonus_rates = process_json_file(json_file)
        all_gacha_rates.extend(gacha_rates)
        all_gacha_bonus_rates.extend(gacha_bonus_rates)
        print(f"✓ ({len(gacha_rates)} rates, {len(gacha_bonus_rates)} bonus rates)")
      except Exception as e:
        print(f"✗ Error:  {e}")
        continue

    # Insert all data in batches
    print(f"\n{'=' * 60}")
    print("Inserting data into database...")
    print('=' * 60)
    insert_data(conn, all_gacha_rates, all_gacha_bonus_rates)

    print(f"\n✓ Data insertion completed successfully!")

  except Exception as e:
    print(f"✗ Error: {e}")
    conn.rollback()
    sys.exit(1)
  finally:
    conn.close()


if __name__ == "__main__":
  main()
