# pgtimewarp CLI

Command-line tool for managing pgtimewarp and querying time travel data.

## Installation

```bash
cargo build --release
```

## Commands

### Track a table

```bash
pgtimewarp track public.orders --pk id --retention 24 --node prod-eu-1
```

### Untrack a table

```bash
pgtimewarp untrack public.orders --node prod-eu-1
```

### List status

```bash
pgtimewarp status
pgtimewarp status --node prod-eu-1
```

### Query as-of

```bash
pgtimewarp asof public.orders --pk id=123 --at "2026-01-13T09:41:00Z" --node prod-eu-1
```

### Show diff

```bash
pgtimewarp diff public.orders --pk id=123 --from "2026-01-13T09:00:00Z" --to "2026-01-13T10:00:00Z" --node prod-eu-1
```

### Health diagnostics

```bash
pgtimewarp doctor
pgtimewarp doctor --node prod-eu-1
```

## Configuration

Set the store database connection via environment variable or flag:

```bash
export PGTIMEWARP_STORE_DSN="postgres://user:password@localhost/pgtimewarp"
pgtimewarp status
```

Or use the flag:

```bash
pgtimewarp status --store-dsn "postgres://user:password@localhost/pgtimewarp"
```
