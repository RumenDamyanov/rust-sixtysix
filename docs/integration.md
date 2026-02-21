# Integration Notes

## Client Architecture

Suggested layers:

1. Transport client (fetch / axios / native) hitting HTTP API.
2. Local state store keyed by session id (Redux / Zustand / Vue store / custom hook).
3. Optimistic updates: append provisional action to a local reducer; replace with authoritative state from response.
4. Re-sync periodically (poll GET /sessions/{id}) or add a push channel (WebSocket) on top.

## Card Rendering

Build a lookup table mapping encoded int -> (suit, rank, points). Avoid recalculating on each render.

## Replays

Store seed + ordered list of actions. Re-simulate by starting from initial state and applying actions sequentially.

## Latency Mitigation

Batch UI events: allow queueing of next intended play but verify after server ack.

## Error Handling

On 400 validation error, roll back optimistic state and surface message to user.

## Persistence Extension

Implement the `Store` trait (create/get/update/list/delete) for PostgreSQL / Redis; register via dependency injection in main.

## Scaling

Stateless API layer behind load balancer; sticky sessions not required because state is persisted via store trait (in-memory replaced by shared backend in production).
