---
name: rust-best-practices
description: >-
  MUST load at every session start in this repo alongside reth-opbnb-port.
  Experienced Rust engineer mindset and Rust best practices for idiomatic,
  safe, performant code. Use for all Rust edits, reviews, refactors, API
  design, error handling, async, unsafe, Clippy/fmt, and Reth crate work.
---

# Experienced Rust Engineer — Best Practices

> Projekt-Skill: `.cursor/skills/rust-best-practices/`.
> Session-Start (zwingend) mit `reth-opbnb-port`: Rule + `sessionStart`-Hook.

Schreibe und review Rust wie ein erfahrener Production-Engineer: klar, idiomatisch,
minimal, messbar. In diesem Repo zusätzlich Reth-Konventionen (`CLAUDE.md` /
`AGENTS.md`: nightly fmt, Clippy, Type-Ordering, Comments-WHY).

## Haltung

1. **Kleine, korrekte Diffs** > große „Aufräum“-PRs. Ein Concern pro Änderung.
2. **Typen tragen die Invarianten** — nicht Kommentare oder Runtime-Checks allein.
3. **Zero-cost wo es zählt**; Allokationen/Clones in Hot Paths vermeiden oder begründen.
4. **Fehler sind Werte** (`Result`/`Option`) — kein `unwrap`/`expect` in Library-/Node-Pfaden außer Tests oder echt unreachable (dann Message).
5. **API so schmal wie nötig** — `pub` nur mit Grund; Prefer `pub(crate)`.

## Idiome (kurz)

| Thema | Praxis |
|-------|--------|
| Ownership | Borrow (`&`/`&mut`) default; `Clone`/`to_owned` nur wenn Lifetime/Ownership es verlangt |
| Iterators | `iter`/`into_iter`/`collect`; rare manual index loops |
| Matching | Exhaustive `match`; `if let` / `let-else` für einen Arm |
| Strings | `&str` in APIs; `String` nur Besitz; `format!` sparsam in Hot Paths |
| Collections | `Vec`/`HashMap` mit Capacity wenn Größe bekannt; `smallvec`/`ArrayVec` nur wenn schon im Crate-Stil |
| Generics | Bounds minimal; `impl Trait` in Argumenten ok; associated types für feste Beziehungen |
| Traits | Object-safe nur wenn Dyn nötig; sonst monomorph |
| Async | `.await` nicht über Sync-Locks halten; CPU/`blocking` → `spawn_blocking`; kein nested runtime |
| Send/Sync | Explizit denken bei Channels/Tasks; `'static` Bounds nur wenn Spawn es braucht |

## Errors

- Library/crate: typed errors (`thiserror` / enum) + `?`; Context an Grenzen (`eyre`/`anyhow` nur wo das Crate es schon so macht).
- Nie leere `map_err(|_| ...)` ohne Information; Preserve `source`/`#[from]`.
- User-/CLI-Grenzen: brauchbare Messages; interne Bugs: `tracing` + typed return.
- Tests: `unwrap`/`expect` ok; Production: nicht.

## Unsafe

- Letztes Mittel. Jeder `unsafe`-Block: **Safety-Kommentar** (Invariante, warum sound).
- Keine raw Pointer-Arithmetik ohne bestehende Crate-Patterns zu spiegeln.
- `unsafe impl Send/Sync`: nur mit dokumentierter Begründung.

## Performance (Reth-relevant)

- Hot Paths (Execution, Trie, Stages, Engine poll): keine unnötigen `clone`, `String`, `collect`→sofort iterieren.
- Parallel: `rayon` wo das Crate es tut; nicht ad-hoc Threads.
- I/O: `reth_fs_util` statt rohem `std::fs` wo vorgeschrieben.
- Messen vor Mikro-Optik außerhalb klarer Hot Paths.

## API- & File-Struktur (Reth)

- **Type ordering:** Primärtyp (Dateiname) zuerst, dann public Aux, Traits, private Helpers — siehe `CLAUDE.md`.
- Neue Traits/Structs **nicht** über dem Primary Type einschieben.
- Feature-Flags: `cfg` sauber; Default-Features nicht leichtfertig ändern.
- Workspace: bestehende Dependency-Versionen/`workspace = true` nutzen; kein wildes Version-Pinning.

## Tooling (dieses Repo)

```bash
cargo +nightly fmt --all
cargo +nightly clippy --workspace --lib --examples --tests --benches --all-features
cargo nextest run -p <crate>
```

- Clippy-Warnungen nicht mit `allow` totstellen ohne Begründung am Attribut.
- `make update-book-cli` nur bei CLI-Flag-Änderungen (nie Hand-Edit an `docs/vocs/.../cli`).

## Review-Checkliste (vor Done)

- [ ] Bounds/Lifetimes so einfach wie möglich?
- [ ] Kein neues `unwrap` in non-test Code?
- [ ] Errors typed + kontextreich an der Grenze?
- [ ] Hot Path: keine offensichtlichen Alloc/Clone-Regressionen?
- [ ] `unsafe` dokumentiert?
- [ ] Öffentliche API/Docs nur wo nötig; Comments erklären **Warum**?
- [ ] Passt zum Crate-Stil (nicht „eigenes Framework“ einführen)?

## Anti-Patterns

- God-functions / 200-Zeilen-`match` ohne Extraktion
- `Arc<Mutex<T>>` wo Channels oder einzelne Ownership reichen
- `Box<dyn Error>` in internen APIs wenn das Crate typed errors hat
- Refactors „weil schöner“ ohne Verify/Test
- Neue Dependencies für Triviales (`once_cell` vs `std::sync::OnceLock`, etc. — std bevorzugen)

## Zusammenspiel

- Port/Sync/opBNB: zuerst **`reth-opbnb-port`** (PORT-PIPE).
- Reiner Rust-Stil/Qualität: dieser Skill.
- Bei Konflikt Port-Korrektheit > kosmetischer Stil — aber neue Diffs trotzdem idiomatisch halten.
