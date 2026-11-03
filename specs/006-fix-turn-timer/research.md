# Research: Fix Turn Timer

## D-601: A timeout's consequence depends on the game type

**Decision**: `turn_timer::expire` takes the round's rule: for an Order round it removes one heart
and passes the turn; for a Tripmine round it only passes the turn (no heart, no points). The
`time_out` handler reads the current round's quiz type and passes the matching rule.

**Rationale**: Hearts are an Order mechanic; a Tripmine round has no heart loss for a wrong pick, so
a timeout there should not invent one. Making the rule explicit keeps the pure function the single
source of the timeout outcome (Principles II and VII). The handler already loads the current round for
its number, so reading its type is the same query.

**Alternatives considered**: leaving a heart loss in both (penalises a Tripmine team that has no
hearts to lose); a separate Tripmine timeout path (duplicates the clear-timer bookkeeping).

## D-602: The timer's cues come from the reflected state, like every other cue

**Decision**: Extend the existing cue derivation. A timer appearing where there was none plays the
start cue (a buzzer, matching the reference draft); a timer clearing with the timeout message plays
the timeout cue (the failure tone). The timeout check runs before the heart-loss check so an Order
timeout plays exactly one cue.

**Rationale**: Feature 004 made all cues server-driven, derived from consecutive states; the timer
must follow the same path so every role hears it and a reconnect replays nothing (Principle II). The
Order timeout already dropped a heart, which would otherwise emit a failure cue; ordering the timer
check first collapses the two into one. The start cue reuses the draft's buzzer.

**Alternatives considered**: a client-side "timer started" hook (a second, non-server cue path, which
feature 004 deliberately removed); a per-second tick sound (noisy and not requested).

## D-603: The timeout is identified by its message

**Decision**: The timeout cue fires when the timer clears and the session's last message is the
timeout message ("Time ran out!..."). Both game types set that message; any other turn end clears the
timer with a different (or no) message and stays silent.

**Rationale**: The state has no dedicated "last event" field, and the timer clears for several
reasons (a placement, a pass, a new round). The timeout is the only one that sets that message, so it
is a reliable, already-present discriminator without a schema or DTO change (Principle VII).

**Alternatives considered**: adding a `lastEvent` field (a realtime-contract change for a small
gain); inferring the timeout from a heart drop (misses Tripmine, which has none).

## D-604: Fixed preset durations

**Decision**: Replace the gamemaster's free-text seconds input with three preset buttons — 15s, 30s,
60s. Starting with a preset while one runs replaces the current timer.

**Rationale**: Presets remove invalid/absurd durations and match the reference draft's fixed choices;
three native buttons are keyboard-operable (Principle VI). The existing `start_timer` action already
accepts any positive seconds, so the server is unchanged (Principle VII).

**Alternatives considered**: keeping the input and validating (still invites odd values); a
dropdown (more interaction than three buttons).
