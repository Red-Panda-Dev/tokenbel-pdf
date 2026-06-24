---
okf_version: "0.1"
---

# Knowledge Bundle

OKF knowledge bundle for the TokenBel PDF pipeline: a Rust workspace that turns
Belarusian statutory financial PDF reports into normalized tabular output
(JSON/XLSX) via OCR, table extraction, and business cleaning.

* [Architecture](architecture/) - logical system shape, dependency direction, and invariants.
* [Pipeline](pipeline/) - the processing stages and external boundaries (OCR, date normalization).
* [Contracts](contracts/) - the CLI JSON contract and the shared domain models.
* [Domain](domain/) - supported report types and normalization rules.
* [Build](build/) - native/wasm target boundaries and feature gating.

See [log.md](log.md) for the chronological update history.
