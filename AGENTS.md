# Repository Guidelines

## Purpose

This repository contains Rust experiments for the CY8CPROTO-062-4343W
PSoC 6 Wi-Fi BT Prototyping Kit. The current active line of work is a
quiet serial-console firmware that will grow into a tiny Lisp machine
with microSD storage and Wi-Fi support.

Keep `README.md` for stable project setup and `bringup.org` for dated,
experiment-level notes, measured hardware facts, toolchain decisions,
and commands that are known to work on the attached board.

## Project Structure

- `psoc6-pac/`: generated peripheral access crate and device metadata.
- `psoc6-cm0-bootloader/`: CM0+ bootloader that prepares and releases
  the CM4 application. Keep it focused on boot, reset, clocks needed for
  boot, and handoff.
- `lisp-psoc-pc/`: current CM4 serial console firmware. This is the
  active place for the tiny Lisp machine work unless a better split is
  deliberately introduced.
- `morse-code*/`, `psoc6-cm*-*`, `simple-db/`: older experiments and
  reference crates. Do not churn them while working on the console Lisp
  machine unless the task explicitly needs it.
- `probe-rs-targets/`: local probe-rs target metadata and compatibility
  notes.
- `backups/`: flash and firmware backups captured before risky changes.
- `bringup.org`: running lab notebook. Update it when a command,
  hardware fact, or implementation decision becomes useful to preserve.

## Programming Policy

No shortcuts, oversimplifications, or cheap hacks. When implementing
features or fixing bugs:

- If asked to provide something that already exists, point it out instead
  of creating an alias or duplicate path.
- Never leave TODOs, FIXMEs, or placeholder implementations.
- Never simplify requirements without explicit approval.
- Never use "good enough for now" as the justification for shipped code.
- Implement the actual complexity when the hardware or runtime requires
  it.
- Prefer established, readable solutions over clever ones.
- Keep code reasonably generic and stratified: do not bury low-level
  register programming inside command parsing, Lisp evaluation, or
  console UX code.
- Prefer smaller functions with descriptive names, even when a helper is
  only used once.
- Do not abbreviate identifiers unless the abbreviation is the hardware
  name from the datasheet, PAC, schematic, or board support package.
- Do not introduce em dashes.

Keep boot files as boot files. Startup and handoff files should contain
startup, mounting, configuration, interrupt/vector setup, and core
handoff logic, not growing command interpreters or unrelated application
behavior.

Split by coherent workflow instead of dumping helpers into one utility
blob. Good splits for the active firmware are console I/O, command
parsing, Lisp reader/evaluator/printer, board peripherals, storage, Wi-Fi,
and diagnostics.

Preserve existing public entry points and console commands where
practical. Move one coherent concern at a time so behavior stays stable
and commits stay reviewable.

## Hardware Safety

Treat hardware writes as externally visible side effects.

- Keep the firmware quiet during unattended work. Do not enable heartbeat
  blinking, run LED-on tests, or flash a deliberately blinking image while
  the user has asked for the LED to stay off.
- LED4 on this board is active-low on `P13.7`. Driving `P13.7` high turns
  it off; driving it low turns it on.
- Confirm pin mappings against the schematic, BSP, or live register
  inspection before programming unfamiliar peripherals.
- Prefer diagnostic commands that report register state without changing
  board state.
- Before risky flash changes, keep or create a backup under `backups/`
  and document the restore path in `bringup.org`.
- Do not use destructive git commands to recover from hardware bring-up
  mistakes.

## Secrets and Local State

- Never commit Wi-Fi credentials, `/var/lib/iwd` contents, tokens, serial
  numbers that are not already public, or generated secret headers.
- Do not print secrets to logs, terminal transcripts, docs, or commit
  messages.
- Any generated credential material must live in an ignored local path.
- Skip enterprise profiles such as eduroam unless the user explicitly asks
  to support them.
- Treat other local checkouts, such as `/root/zmk` or `/root/project/hiisi`,
  as read-only references unless the user explicitly asks to edit them.

## Rust Style

- Format Rust code with `cargo fmt` in the crate being changed.
- Do not leave unrelated rustfmt spillover unstaged and unaccounted for.
  Include deliberate formatting changes in the commit that needs them, or
  avoid touching unrelated crates.
- Use `snake_case` for variables and functions, `CamelCase` for types, and
  descriptive names for hardware helpers.
- Keep `unsafe` and raw register manipulation narrowly scoped and named
  after the hardware behavior being established.
- Add succinct comments only where the register sequence or hardware
  assumption is not self-explanatory.

## Build, Test, and Flash

After any code, docs, or config change, run the smallest validation that
proves the changed surface is sane before committing. For docs-only
changes, `git diff --check` is enough unless the docs include commands
that were changed and should be tested.

For the active console firmware:

```sh
cd lisp-psoc-pc
RUSTFLAGS=-Awarnings cargo build --release --features use-bootloader
arm-none-eabi-objcopy -O binary \
  target/thumbv7em-none-eabihf/release/lisp-psoc-pc \
  ../psoc6-cm0-bootloader/src/app.bin

cd ../psoc6-cm0-bootloader
RUSTFLAGS=-Awarnings cargo build --release

cd ..
/opt/ModusToolbox/tools_3.4/openocd/bin/openocd \
  -s /opt/ModusToolbox/tools_3.4/openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/psoc6_2m.cfg \
  -c "program psoc6-cm0-bootloader/target/thumbv6m-none-eabi/release/psoc6-cm0-bootloader verify reset exit"
```

OpenOCD's `target/psoc6_2m.cfg` warning is known. Keep using the working
target file until a replacement command is tested and documented.

## Commit Policy

- Commit messages use primitive style, title line only, under 72
  characters.
- No `Co-Authored-By` lines.
- Prefer rebase over merge; avoid merge commits.
- Commit after each completed logical change.
- Push after each commit when a remote is configured and the network/key
  setup allows it.
- Keep commits tiny, granular, and single-purpose.
- Do not bundle unrelated regressions, unrelated experiments, or broad
  refactors into one commit just because the files are nearby.
- Prefer one commit per regression fix, one commit per feature slice, and
  one commit per logical file-splitting group.
- If a task is broad, land it as a sequence of small vertical commits
  instead of one final polish commit.
- Do not include unrelated dirty worktree changes in a commit.
- Add short, dated entries to `bringup.org` for notable hardware,
  toolchain, or runtime behavior decisions.

## Worktree Discipline

The worktree may contain user changes. Never revert, overwrite, or stage
changes you did not make unless the user explicitly asks for that exact
operation. If unrelated dirty files exist, leave them alone. If user
changes affect the active task, work with them and preserve their intent.

Use non-interactive git commands where practical. Avoid destructive
commands such as `git reset --hard` or `git checkout --` unless the user
explicitly asks for them.
