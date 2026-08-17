# Anonymized milestone publish (`alteredcarbon`)

Local `rebase/reth-v2.4.1` keeps full history. GitHub `alteredcarbon/main` gets a short
orphan log of scrubbed trees (no host/IP/path/infra identifiers).

Milestones include **protocol/sync gates** and **AI experiment effort** (session telemetry,
illustrative LLM cost, operator/senior-admin human-owned work) — see README *Effort log*.

```bash
# rebuild only
scripts/publish/build_alteredcarbon_milestones.sh

# rebuild + force-with-lease push to alteredcarbon/main
scripts/publish/build_alteredcarbon_milestones.sh --push
```

Scrub rules: `scrub_public_tree.py` (IPs, absolute paths, CT/datadir names, nspawn, …).
Keeps AI tool names and upstream software (reth, op-geth, …).
