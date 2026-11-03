# Research: Fix Timer Timeout

## D-801: The server owns the deadline and applies the timeout itself

**Decision**: When a timer starts, the server schedules the timeout for its end time and applies it
there, on the runtime the server already runs. The timeout is applied by one named rule
(`sessions::apply_turn_timeout`), reused by the manual `time_out` action.

**Rationale**: The current design relies on a client posting `time_out` at zero, but a background tab
throttles its timers (to roughly once a minute), and even a foreground post can be rejected a moment
early as "not yet expired". So the turn often never ends, and the rejection shows the stray
`"The timer has not run out yet."` bar. A server-owned deadline ends the turn regardless of any client,
which is what the spec requires ("without further user action"). The rule itself is unchanged from
feature 006 (Order: one heart + pass; Tripmine: pass), so only who triggers it changes
(Principles II and III).

**Alternatives considered**: keeping the client post and retrying silently (still fails with a closed
or throttled tab); a coarse poll on the SSE heartbeat (15 s late — unacceptable for a 15 s timer).

## D-802: The scheduled timeout is safe against stop and replace

**Decision**: The scheduled task records the end time it was created for; when it wakes it applies the
timeout only if the session still has a timer with that same end time. Stopping the timer, starting a
new one, ending the round, or deleting the session makes it a no-op. The rule is also idempotent (no
timer → nothing).

**Rationale**: A timer can be stopped or replaced before it fires; the task must not penalise a turn
that no longer has that timer. Comparing the stored end time is the simplest reliable guard
(Principle VII).

**Alternatives considered**: cancelling the task on stop (needs task handles stored per session —
more state for no benefit); a generation counter (heavier than the end-time comparison).

## D-803: The client no longer posts the timeout

**Decision**: Remove the client's `time_out` post and its error handling. The client keeps the
countdown for display and ticks; the turn ends when the server applies the timeout.

**Rationale**: The client post is the source of both the stray bar and the unreliable turn end. With
the server owning the deadline the client has nothing to post, so the error path disappears
(FR-005) and the turn end is reliable (FR-001). The `time_out` action stays for manual/testing use.

**Alternatives considered**: keeping the post and suppressing its error (still double work and
unreliable); hiding the error only (leaves the turn ending unreliable).

## D-804: Ticks are client-side, on the last five seconds

**Decision**: The client plays a short tick on each of the last five seconds of the countdown
(seconds 5, 4, 3, 2, 1; a timer shorter than five seconds ticks on each remaining second), reusing an
existing short cue. The start cue and the timeout cue are unchanged.

**Rationale**: The countdown is already a client-side display of the server's end time, so the
per-second ticks belong with it; they are a local highlight, not a game event. Reusing an existing
short cue avoids new audio (Principle VII). Ticks obey mute through the existing play path.

**Alternatives considered**: server-pushed tick events (needless traffic for a display detail);
a new synthesized effect (unnecessary).
