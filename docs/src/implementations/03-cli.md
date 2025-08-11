# Implementation: 03 - Command-Line Interface (`elfi-cli`)

This document specifies the desired commands, arguments, and functionality for the `elfi-cli` binary crate. The CLI serves as a tool for offline file management, validation, and conversion.

We will use the `clap` crate with its "derive" feature for parsing arguments.

## Commands

### `elfi new <PATH>`

Creates a new, minimal `.elf` file at the specified path.

-   **Arguments**:
    -   `<PATH>`: The path where the new file should be created.
-   **Behavior**:
    -   Generates a single, empty `markdown` block.
    -   Assigns a new `UUID` to the block's `id`.
    -   Writes the resulting text to the specified file.

### `elfi validate <PATH>`

Validates the structure and syntax of an `.elf` file.

-   **Arguments**:
    -   `<PATH>`: The path to the `.elf` file to validate.
-   **Behavior**:
    -   Reads the file content.
    -   Runs it through the `elfi-parser`.
    -   If parsing is successful, it prints a success message and exits with code 0.
    -   If parsing fails, it prints a detailed error message (e.g., YAML parsing error, Tree-sitter syntax error) and exits with a non-zero code.

### `elfi export <PATH>`

Parses an `.elf` file and exports its current state to a different format.

-   **Arguments**:
    -   `<PATH>`: The path to the `.elf` file.
-   **Options**:
    -   `--format <FORMAT>`: The output format. Defaults to `json`.
        -   `json`: Exports the full Automerge document as a single JSON object.
        -   `raw-json`: Exports the document in the Tangle-API-compatible format (see `02-core_logic.md`).
-   **Behavior**:
    -   Parses the `.elf` file into an in-memory Automerge document.
    -   Serializes the document to the specified format.
    -   Prints the result to standard output.
