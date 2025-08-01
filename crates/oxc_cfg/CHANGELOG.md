# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).













# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.71.0] - 2025-05-20

### Refactor

- bb8bde3 various: Update macros to use `expr` fragment specifier (#11113) (overlookmotel)

## [0.63.0] - 2025-04-08

### Performance

- fa0e455 cfg, diagnostics, lexer, syntax, tasks: Remove `write!` macro where unnecessary (#10236) (overlookmotel)

### Styling

- 66a0001 all: Remove unnecessary semi-colons (#10198) (overlookmotel)

## [0.52.0] - 2025-02-21

### Bug Fixes

- 358320d cfg: Fix `clippy::debug_assert_with_mut_call` (#9257) (Boshen)

### Refactor

- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)

## [0.49.0] - 2025-02-10

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.37.0] - 2024-11-21

### Features

- 8cfea3c oxc_cfg: Add implicit return instruction (#5568) (IWANABETHATGUY)

## [0.31.0] - 2024-10-08

- 95ca01c cfg: [**BREAKING**] Make BasicBlock::unreachable private (#6321) (DonIsaac)

### Features

- fa4d505 cfg: Derive more base traits for CFG blocks (#6320) (DonIsaac)
- 14275b1 cfg: Color-code edges in CFG dot diagrams (#6314) (DonIsaac)

### Refactor

- 40932f7 cfg: Use IndexVec for storing basic blocks (#6323) (DonIsaac)
- a1e0d30 cfg: Add type alias for Graph (#6322) (DonIsaac)
- 7672793 cfg: Move block data types to separate file (#6319) (DonIsaac)

## [0.29.0] - 2024-09-13

### Refactor

- cc0408b semantic: S/AstNodeId/NodeId (#5740) (Boshen)

## [0.21.0] - 2024-07-18

### Refactor

- fc0b17d syntax: Turn the `AstNodeId::dummy` into a constant field. (#4308) (rzvxa)

## [0.20.0] - 2024-07-11

### Bug Fixes

- 7a059ab cfg: Double resolution of labeled statements. (#4177) (rzvxa)

## [0.16.0] - 2024-06-26

### Features

- 3e78f98 cfg: Add depth first search with hash sets. (#3771) (rzvxa)

## [0.15.0] - 2024-06-18

- 0537d29 cfg: [**BREAKING**] Move control flow to its own crate. (#3728) (rzvxa)

### Refactor

- d8ad321 semantic: Make control flow generation optional. (#3737) (rzvxa)

