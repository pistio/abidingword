"""Offline #35 query-plan/work regression; invented metadata, no PDF content.

Reads the production query, using schema-27 index names. This isolates SQLite
work; the Rust 400-page regression covers the actual transaction and rollback.
"""
import json
from pathlib import Path
import re
import sqlite3
import time

source = Path('src-tauri/src/db/pdf_structures.rs').read_text()
query = re.search(r'conn\.prepare\("(SELECT segment_id,id,start_offset,end_offset,structure_role[^"\n]+)"\)', source).group(1)
db = sqlite3.connect(':memory:')
db.executescript('''
CREATE TABLE source_passages(id TEXT PRIMARY KEY,source_id TEXT,segment_id TEXT,
 origin TEXT,start_key INTEGER,end_key INTEGER,start_offset INTEGER,end_offset INTEGER,structure_role TEXT);
CREATE INDEX source_passages_source ON source_passages(source_id);
CREATE INDEX source_passages_segment ON source_passages(segment_id);
CREATE INDEX source_passages_origin_range ON source_passages(origin,start_key,end_key);
''')
db.executemany('INSERT INTO source_passages VALUES(?,?,?,?,0,0,0,4,NULL)',
               [(str(i), 'fixture', f'segment-{i % 2719}', 'explicit') for i in range(4112)])
plan = [r[3] for r in db.execute('EXPLAIN QUERY PLAN ' + query, ['fixture'])]
assert any('source_passages_source (source_id=?)' in row for row in plan), plan
work = [0]

def tick():
    work[0] += 100
    return 0

start = time.perf_counter()
db.set_progress_handler(tick, 100)
rows = list(db.execute(query, ['fixture']))
db.set_progress_handler(None, 0)
assert len(rows) == 4112
assert work[0] < 100_000, f'Source-scoped fetch used too many SQLite VM steps: {work[0]}'
print(json.dumps({'queryPlan': plan, 'links': len(rows), 'approxVmSteps': work[0],
                  'elapsedMs': round((time.perf_counter() - start) * 1000, 2)}, indent=2))
