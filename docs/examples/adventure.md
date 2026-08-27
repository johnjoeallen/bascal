[Home](../../) / [Examples](sort-driver.md) / Adventure Game

<div class="prose" markdown="1">

# Case Study: Huw Collingbourne's Adventure Game

This [console port](https://github.com/johnjoeallen/bascal/tree/main/examples/adventure) of `Delphi/game02` comes from Huw Collingbourne's Adventure
Games source archive. The original is a VCL application. Its form is replaced
by a command loop, but its six-room map, room descriptions, exits, player,
inventory, take/drop behaviour, and save/load state are preserved.

The port is deliberately written for BASCAL's planned full record model, not
for a restricted BASIC backend. It is therefore a language-design case study:
the source is the desired BASCAL program, while current backend gaps are
implementation work rather than constraints on the design.

## Current status

The complete multi-file program **compiles through the BASCAL front end**:

```text
bcc examples/adventure/main.bcl -L examples --check
```

`--check` loads all dotted `require` paths and parses the complete program,
but deliberately stops before record lowering, resolution, and code
generation. No current BASIC, C, or JVM backend can generate this program
yet. This is intentional: the example is a conformance case for the planned
record language surface, rather than a claim of backend support.

The planned facilities exercised by the port are:

- general-purpose record declarations and record variables;
- typed arrays of records and typed record parameters, including `byref`;
- nested record fields and nested field access;
- record literals, value assignment, and mutable fields;
- type-scoped `method[Type]` declarations and `self`;
- typed record method results;
- fluent record mutation; and
- record method calls on record values and array elements.

The conformance report lists this as a successful **front-end-only** check and
marks all three backends `UNIMPLEMENTED` until a backend can generate the
required planned record semantics.

Run it, once a backend supports the planned record model, with:

```text
bcc examples/adventure/main.bcl -L examples --target c --run
```

The console accepts `N`, `S`, `E`, `W`, `NORTH`, `SOUTH`, `EAST`, `WEST`,
`LOOK`, `TAKE <thing>`, `DROP <thing>`, `INVENTORY`/`I`, `SAVE`, `LOAD`, and
`QUIT`.

## Delphi-to-BASCAL mapping

| Delphi concept | BASCAL representation | Assessment |
| --- | --- | --- |
| `Thing` | `Thing` record with name, description, and location | ACCEPTABLE |
| `ThingHolder` | location field plus shared lookup/move procedures | ACCEPTABLE |
| `Room` | `Room` record containing an `Exits` record | ACCEPTABLE |
| `Actor` | `Actor` record with current room | ACCEPTABLE |
| `ThingList` / `RoomList` | fixed arrays of records plus constants | ACCEPTABLE for this fixed world |
| inheritance | composition plus small shared procedures | ACCEPTABLE |
| object references | stable room IDs and thing locations | ACCEPTABLE |
| constructors | record literals and `build_world` | ACCEPTABLE |
| virtual stream methods / `TFileStream` | explicit sequential state serialization | ACCEPTABLE |
| VCL form/buttons | console command loop | deliberately replaced |

## Features that mapped cleanly

Records map directly to rooms, exits, things, and the player. Nested `Exits`
records preserve the original room topology without flattening it. Type-scoped
methods keep `Room.exit`, `Room.describe`, and `Actor.moveTo` with the state
they operate on. Record literals make world construction concise; the fluent
`Room.setExit` method is exercised by the model tests.

The original inheritance hierarchy mainly contributes common name/description
fields and a `TList` of contained objects. This game does not need runtime
subtype dispatch once its world is represented as record arrays, so BASCAL does
not need inheritance to express it clearly.

## Features that are different but acceptable

Room IDs replace Delphi object references. The map already used integer exits,
so `rooms%(ROOM_0).exits.east = ROOM_1` remains explicit and readable. A
thing's location is a room ID or the `INVENTORY` sentinel; moving an item is a
single field update rather than removing an object from one `TList` and adding
it to another.

Save/load persists logical state—the player room and thing locations—rather
than Delphi's binary object graph. That is smaller, portable, and sufficient
because `build_world` reconstructs immutable names, descriptions, and exits.

## Awkward BASCAL areas

### Requirement: a reusable mutable collection

**Current BASCAL solution:** fixed record arrays, a fixed count, and a
location field.

**Why it is awkward:** it is clean for `game02`, whose six rooms and ten
things are fixed, but a content-authoring version with arbitrary rooms or a
growing inventory would require manual capacity and count management.

**Possible language improvement:** a standard growable typed list with
append/remove and iteration. This is AWKWARD, not MISSING, for this program.

### Requirement: command tokenisation

**Current BASCAL solution:** `trim`, `upper`, `left`, and `mid` recognise the
small `TAKE`/`DROP` command grammar.

**Why it is awkward:** the approach is adequate here but becomes repetitive
for richer verbs, articles, and multiple noun phrases.

**Possible language improvement:** a standard string `split`/token iterator.
This is AWKWARD, not MISSING.

## Missing BASCAL capabilities

No additional language feature is required to express the `game02` behaviour
under the assumed general-purpose record model. The front end accepts the
planned syntax under `--check`, but current code-generation backends do not
yet implement general records, record arrays, nested records, record methods,
typed record parameters, record literals/value semantics, or fluent record
mutation. These are implementation gaps, not missing language-design
capabilities under this case study's stated assumptions.

## Delphi facilities that were not necessary

The VCL GUI, constructors/destructors, virtual persistence methods, `TList`,
and the `ThingHolder` inheritance layer are all reasonable Delphi choices, but
not semantic requirements of this game. The BASCAL version is procedural at
the coordination layer and uses record methods only where a record's own state
is involved.

The complete port and its model-level tests are in
[`examples/adventure`](https://github.com/johnjoeallen/bascal/tree/main/examples/adventure).

</div>

[← Card Catalog](card-catalog.md)
