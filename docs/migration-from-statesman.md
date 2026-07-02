# Migration From Statesman

Automatic Convex migration is out of scope for the standalone local CLI. Port records by hand into canonical JSON or JSONL, then run `provenance import` and `provenance check`.

The local format intentionally avoids hosted auth, user tables, and work queues.
