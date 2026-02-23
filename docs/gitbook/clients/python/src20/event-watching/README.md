---
description: Polling SRC20 event watchers with decryption
icon: radar
---

# Event Watching

SRC20 watchers poll logs, decrypt encrypted amounts, and invoke callbacks for Transfer/Approval events.

## Factory functions

- [watch_src20_events](watch-src20-events.md): fetch viewing key from Directory
- [watch_src20_events_with_key](watch-src20-events-with-key.md): use explicit viewing key

Each has async variants.

## Watcher classes

- [SRC20EventWatcher](src20-event-watcher.md)
- [AsyncSRC20EventWatcher](async-src20-event-watcher.md)

Factory functions start watchers before returning.
