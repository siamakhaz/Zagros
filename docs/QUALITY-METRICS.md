# Quality Metrics

Zagros uses deterministic retrieval regression tests and corpus health metrics to catch quality regressions before release.

## Retrieval benchmark

The checked-in benchmark lives under `benchmarks/retrieval/` and is executed by `tests/retrieval_benchmark.rs`.

Current baseline:
- 32 total query cases
- exact and semantic CVE retrieval
- CWE, CAPEC, ATT&CK, and ASVS retrieval
- irrelevant-query rejection

The regression gate requires:
- Recall@1 >= 0.90
- Recall@5 >= 0.98
- MRR >= 0.93
- irrelevant-query rejection >= 0.80
- at least 30 benchmark cases

The corpus and expected answers are intentionally small, deterministic, offline fixtures. They are not a substitute for future large-corpus evaluation, but they prevent accidental scoring/tokenization regressions.

Run locally:

```powershell
cargo test --test retrieval_benchmark -- --nocapture
```

The normal `cargo test --all-targets` CI gate also executes this benchmark.

## Corpus integrity and freshness

`index_status` now returns per-source health for CVE, CWE, ASVS, CAPEC, and ATT&CK.

Each source reports:
- current record count
- conservative expected minimum count
- integrity: `healthy`, `degraded`, or `empty`
- latest provenance retrieval timestamp
- retrieval age in hours
- freshness: `fresh`, `stale`, or `unknown`

Current conservative minimums are CVE 400, CWE 900, ASVS 300, CAPEC 500, and ATT&CK 600. These are regression tripwires, not assertions that upstream source counts must remain exact.

A source is fresh when its newest successful provenance retrieval is no more than 48 hours old. Missing provenance produces `unknown` freshness.

`overall_health` is `healthy` only when every tracked source has healthy integrity and fresh provenance. Otherwise it is `degraded`.

The 48-hour window intentionally allows one missed daily refresh without immediately declaring the corpus stale, while still surfacing a failed refresh cycle promptly.

## Future evolution

P2-C2 establishes a lexical retrieval and corpus-health baseline. Future vector/graph/hybrid retrieval must meet or improve the benchmark before stabilization. Larger real-world benchmark sets can be added without weakening the existing regression thresholds.
