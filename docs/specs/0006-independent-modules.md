# 0006. Independent module storage

**Date**: 2026-09-24
**Status**: In Progress

## Summary

Canvas, Docs, Notes and Calendar keep separate local data. Canvas projects contain only Canvas data. Docs, Notes and Calendar stay available even when no Canvas project exists.

## Requirements

- **AC-1**: Creating or deleting a Canvas project never changes Docs, Notes or Calendar data.
- **AC-2**: Docs, Notes and Calendar open, list and create content without a Canvas project.
- **AC-3**: Existing module records migrate without changing their IDs or deleting the source records.
- **AC-4**: Canvas offers an existing project picker and an in place action to create a Canvas project.

## Decision

Use one SQLite database per independent module in the Orbit application data directory. `projects/<id>/project.db` remains Canvas only.

## Feature design

| Module | Database | Entities |
|---|---|---|
| Canvas | `projects/<id>/project.db` | project metadata, canvases, elements, connections, assets |
| Docs | `docs.db` | spaces, documents, document assets |
| Notes | `notes.db` | notes, note assets |
| Calendar | `calendar.db` | entries |

The one time migration copies records from every active and trashed Canvas project with `INSERT OR IGNORE`, retaining legacy source records for recovery. Module APIs take no Canvas project ID.

## Build plan

1. Add module databases and idempotent copy migration, satisfies **AC-1**, **AC-3**.
2. Route Docs, Notes and Calendar through module commands and project free routes, satisfies **AC-1**, **AC-2**.
3. Make templates Canvas only and add Canvas project selection, satisfies **AC-1**, **AC-4**.
4. Add migration and isolation tests, satisfies **AC-1**, **AC-3**.

## Consequences

The user keeps independent knowledge without an artificial Canvas project. Existing legacy project databases remain as a recoverable duplicate until a future explicit cleanup policy.

## Migration plan

**Strategy**: strangler

1. Create module databases and copy legacy rows transactionally.
2. Switch UI and commands to the new databases.
3. Keep legacy rows untouched for rollback and recovery.
