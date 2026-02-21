# API Guide

High-level summary; for schema details see the OpenAPI spec.

## Sessions

Create session:

```http
POST /sessions?game=sixtysix&seed=42
```

Response JSON (abbrev):

```json
{
  "id": "abc123",
  "gameName": "sixtysix",
  "version": 1,
  "state": { }
}
```

List sessions:

```http
GET /sessions?game=sixtysix&offset=0&limit=20
```

Get session:

```http
GET /sessions/{id}
```

Apply action:

```http
POST /sessions/{id}
Content-Type: application/json

{"type":"play","payload":{"card":211}}
```

Delete:

```http
DELETE /sessions/{id}
```

## Actions

| Type | Payload | Notes |
|------|---------|-------|
| play | {card:int} | Plays a card; resolves trick after two plays |
| closeStock | - | Only when stock open and not mid-trick |
| declare | {suit:int} | Marriage (K+Q) at start of trick only |
| exchangeTrump | - | 9 of trump swap; stock open; start of trick |

## Versioning

Each action increments `session.version`. Clients should treat the response as canonical state.

## Errors

Returned as HTTP 400 with error message for validation issues.

## Determinism

Supplying the same seed yields identical initial hands and trump.
