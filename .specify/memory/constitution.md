# ListQuiz Constitution

## Core Principles

### I. Independently Testable Features

Every feature MUST be deliverable and verifiable as a unit, with its tests runnable
without standing up unrelated features, network access, or external services. The
SQLite session store lives in the project tree and MUST NOT require network access
to exercise. A feature that can only be exercised through the full application is
not finished.

**Rationale**: feature-shaped test boundaries keep failures local and let features
land independently instead of in one big-bang integration.

### II. Explicit Business Logic

All business rules MUST exist as named, directly callable units of code — not
inlined into handlers, templates, or UI event callbacks. No rule may live only in
a framework callback or a data-access layer. Every business rule MUST be covered
by a test.

**Rationale**: logic that is only reachable through the UI cannot be unit tested,
reused, or reasoned about in isolation.

### III. Rust Backend

The backend MUST be written in Rust. No second backend language may be introduced.

**Rationale**: one backend language keeps the type system, error model, and test
toolchain uniform across the whole backend.

### IV. Backend-Owned Frontend

The application build MUST export the frontend and the backend MUST serve that
exported output. The frontend MUST NOT be runnable as a standalone application
against nothing; a frontend running without its backend is a broken state, not a
supported mode.

**Rationale**: frontend and backend form one deployable unit. A split runtime
invites a class of bugs that cannot occur when the backend always owns delivery.

### V. Small Independent Modules

Each feature or function MUST live in its own small module with a single clear
responsibility. Modules MUST NOT reach into each other's internals; cross-module
communication goes through explicit, narrow interfaces. New features MUST be
addable without editing existing feature modules.

**Rationale**: module isolation is what makes the system extensible; coupling
turns every change into a rewrite.

### VI. Responsive and Keyboard Accessible UI

The UI MUST stay responsive during backend work — no input blocking, no frozen
frame, no full-page reload for a local state change. Every interactive control
MUST be reachable and operable by keyboard alone, with visible focus indication.

**Rationale**: accessibility and responsiveness are correctness properties, not
polish; they are cheapest to guarantee when treated as acceptance criteria.

### VII. Simplicity Over Abstractions

Prefer the simplest solution that fully satisfies a requirement. Abstraction
layers, indirection, generic frameworks, and speculative extension points MUST
NOT be introduced without a concrete present-day requirement they demonstrably
serve. Adding a layer requires naming the requirement that needs it. This is the
"do not over-engineer" rule: simplicity is a hard constraint, not a preference.

**Rationale**: unused abstractions cost maintenance forever and are never
justified by future hypothetical needs.

## Technical Stack & Build Constraints

- **Backend**: Rust only.
- **Frontend location**: frontend source MUST live inside this repository, in a
  folder owned by this project. The concrete directory is chosen by the plan for
  the feature being built, not by this constitution.
- **`the reference frontend` is the orientation reference, read-only**: the frontend MUST be
  oriented at `the reference frontend`. It may be read, consulted, and used as a visual or
  structural reference. Its files MUST NOT be edited, and it MUST NOT be used as
  the frontend's directory, as an import source, or as a build input. Design cues
  taken from it MUST be reimplemented inside this repository, not referenced
  from it.
- **Build pipeline**: the app build compiles the frontend, exports it, and the
  backend serves the exported result.
- **Build locality**: all compilation MUST happen inside the project folders.
  Writing build output or tool state under `%APPDATA%` or any other AppData
  location is forbidden.
- **Build artifacts are ignored**: whichever folder the build exports into MUST
  be listed in `.gitignore` and MUST stay untracked. Adding the entry is part of
  scaffolding that folder, not a follow-up chore.
- **Session storage**: session data lives in a SQLite database whose file MUST
  sit in the project tree. Session passwords are not treated as secrets and MUST
  NOT be stored hashed or otherwise protected. No encryption, salting, or key
  management is required for them.

**Rationale**: pinning the stack and build behavior keeps the build reproducible
and prevents state leaking outside the repository. Keeping `the reference frontend` read-only
keeps this project's sources self-contained and buildable without a sibling
checkout, while still letting it serve as the design reference.

## Development Workflow

1. Orient at `the reference frontend` read-only before frontend work; reimplement the cues
   inside this repository.
2. Implement one feature per module, following Principle V.
3. Express the feature's business rules as named, tested units, per Principle II.
4. Write tests for that feature that run in isolation, per Principle I.
5. Verify responsive behavior and keyboard operability for any UI change, per
   Principle VI.
6. Keep the implementation to the simplest form that passes, per Principle VII.
7. Before committing, confirm build output stayed inside the project folders and
   that the build artifact folder is listed in `.gitignore` and untracked.

Any deviation from a Core Principle MUST be justified in writing at review time.

## Governance

This constitution supersedes all other team practices and conventions. Where a
plan, spec, or habit conflicts with a Core Principle or a stack constraint, the
constitution wins and the conflict must be corrected rather than worked around.

**Amendments**: Amend this file through the constitution update workflow. Each
amendment MUST bump the version, record the date, and state which principles
changed and why. Amendments that invalidate existing code MUST carry a migration
plan.

**Versioning** follows semantic versioning:
- **MAJOR** — a principle or non-negotiable constraint is removed or redefined
  incompatibly, including reversing a previously permitted action.
- **MINOR** — a principle or section is added, or existing guidance is materially
  expanded without reversing prior permission.
- **PATCH** — clarifications, wording fixes, and non-semantic refinements.

**Compliance**: Every review MUST verify that changed code obeys all seven Core
Principles, the stack and build constraints, and the workflow. Complexity that
violates Principle VII MUST be rejected unless the reviewer accepts the named
requirement justifying it. Any diff touching `the reference frontend`, or any build artifact
found tracked, MUST be rejected.

**Version**: 2.1.1 | **Ratified**: 2026-10-04 | **Last Amended**: 2026-10-04