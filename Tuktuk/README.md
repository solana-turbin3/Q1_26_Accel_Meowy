# TukTuk Assignment

## Assignment

Implement the scheduler and the cron job in any of the previous challenges.

## Implementation

TukTuk is integrated into the **[escrow-litesvm](../escrow-litesvm)** program as an **auto-refund scheduler**. When a maker creates an escrow, they can schedule a TukTuk task that automatically refunds the escrowed tokens if no taker fulfills the trade by the expiry time.

### New Instructions

**`auto_refund`** — Permissionless refund executed by TukTuk crankers
- No maker signature required (TukTuk crankers call this on behalf of the maker)
- Graceful no-op if the escrow was already taken or manually refunded
- Transfers vault tokens back to maker and closes both vault and escrow accounts

**`schedule_refund`** — Maker-signed instruction to schedule the auto-refund
- CPIs into TukTuk's `queue_task_v0` to register a time-triggered task
- Compiles the `auto_refund` instruction into TukTuk's transaction format
- Accepts an `expiry_timestamp` to set when the refund should execute

### Source Files

- [`programs/anchor-escrow/src/instructions/auto_refund.rs`](../escrow-litesvm/programs/anchor-escrow/src/instructions/auto_refund.rs)
- [`programs/anchor-escrow/src/instructions/schedule_refund.rs`](../escrow-litesvm/programs/anchor-escrow/src/instructions/schedule_refund.rs)

### Tests

All 7 tests pass including 2 new auto-refund tests:
- `test_auto_refund_success` — make → auto_refund → verify tokens returned, vault + escrow closed
- `test_auto_refund_already_taken_noop` — make → take → auto_refund → verify graceful no-op

```bash
cd ../escrow-litesvm
cargo test -p anchor-escrow -- --nocapture
```

### Resources

- [TukTuk Docs](https://www.tuktuk.fun/docs)
- [Helper Repo](https://github.com/ASCorreia/tuktuk-counter)
- [Cron schedule expressions](https://crontab.guru/)
