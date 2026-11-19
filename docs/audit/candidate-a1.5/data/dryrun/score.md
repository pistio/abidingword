| Measure | Value |
|---|---:|
| Documents | 5 |
| **Verified, wrong text** | **0** |
| **Verified, not in the edition** | **0** |
| **Verified in controls** | **0** |
| Verified, correct | 33548 |
| Refused, correct | 234 |
| Caught | 176 |
| No result (crash, timeout, killed) | 0 |
| Refused whole document | 1 |
| Highest reading peak, MB | 249 |

| Format | Docs | Correct | Wrong | Absent | Control verified | Refused | Caught | No result |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| docx | 1 | 3354 | 0 | 0 | 0 | 9 | 0 | 0 |
| epub | 2 | 30194 | 0 | 0 | 0 | 225 | 176 | 0 |
| md | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| pdf | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

| Language | Docs | Correct | Wrong | Absent | Control verified | Refused | Caught | No result |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| deu | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| en | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| eng | 1 | 5889 | 0 | 0 | 0 | 200 | 0 | 0 |
| por | 1 | 24305 | 0 | 0 | 0 | 25 | 176 | 0 |
| spa | 1 | 3354 | 0 | 0 | 0 | 9 | 0 | 0 |

| Script | Docs | Correct | Wrong | Absent | Control verified | Refused | Caught | No result |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Latin | 5 | 33548 | 0 | 0 | 0 | 234 | 176 | 0 |

| Kind | Docs | Correct | Wrong | Absent | Control verified | Refused | Caught | No result |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| control | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| generated | 4 | 33548 | 0 | 0 | 0 | 234 | 176 | 0 |

| Document | Result | Correct | Wrong | Absent | Refused | Caught | Peak MB | Exit |
|---|---|---:|---:|---:|---:|---:|---:|---|
| `dry-G01` | scored | 24305 | 0 | 0 | 25 | 176 | 249 | 0 |
| `dry-G05` | refused_document |  |  |  |  |  |  | 0 |
| `dry-G08` | scored | 3354 | 0 | 0 | 9 | 0 | 35 | 0 |
| `dry-G12` | scored | 5889 | 0 | 0 | 200 | 0 | 40 | 0 |
| `dry-control` | control |  | 0 |  |  |  | 18 | 0 |
