# Implementation Design

This section translates the high-level architectural principles from the preceding chapters into a more concrete and actionable implementation plan for the `elfi` Rust kernel.

These documents are intended to be living documents that will evolve as the implementation progresses. They serve as a guide for developers, outlining the core data structures, workflows, and API contracts that will form the foundation of the system.

The following documents detail the implementation specifics for each major component of the `elfi` workspace:

- **[01 - Parser (`elfi-parser`)](./01-parser_and_format.md)**: Defines the initial plain-text format for `.elf` files and the strategy for parsing it into a CRDT model.

- **[02 - Core Logic (`elfi-core`)](./02-core_logic.md)**: Details the internal data structures, asynchronous workflows, and API implementations for the core library.

- **[03 - Command-Line Interface (`elfi-cli`)](./03-cli.md)**: Specifies the commands, arguments, and functionality of the command-line tool.
