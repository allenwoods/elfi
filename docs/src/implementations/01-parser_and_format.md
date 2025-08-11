# Implementation: 01 - Parser and .elf Format

This document specifies the initial plain-text format for `.elf` files and the strategy for parsing this format into the CRDT-based in-memory model.

## 1. `.elf` Plain Text Format

The format is designed to be human-readable, version-control-friendly, and easily parsable. It consists of a sequence of **Blocks** separated by a standard `---` separator. This design is directly inspired by the provided `example.elf.md`.

A **Block** has two distinct parts:
1.  A **Metadata Section** (YAML Frontmatter)
2.  A **Content Section**

### Example Block Structure:

```
---
id: block-C
type: code
metadata:
  parent: block-B
  language: python
---
import pandas as pd
penguins = sns.load_dataset("penguins")
penguins.head()
```

### Metadata Section

-   **Syntax**: A valid YAML object enclosed by `---` at the beginning of a block.
-   **Required Fields**:
    -   `id` (String): A unique identifier for the block. While any string is valid, using UUIDs is recommended for new blocks to prevent collisions.
    -   `type` (String): A string literal defining the block's type (e.g., `"markdown"`, `"code"`). This determines how the content is rendered and processed.
-   **Optional Fields**:
    -   `metadata` (Object): A nested YAML object for storing arbitrary metadata. This provides a namespace for extensibility and is the required location for semantic information like:
        -   `parent` (String): The `id` of the parent block, used to construct the logical hierarchy.
        -   `language` (String): The language of a `code` block (e.g., `"python"`, `"rust"`).
        -   `interactive` (Boolean): A flag for Tangle to determine if a block should be "hydrated" as an interactive island.

### Content Section

The content section contains the raw text of the block. It begins immediately after the closing `---` of the metadata section and extends until the next block separator (`---`) or the end of the file.

## 2. Parsing Strategy (`elfi-parser`)

The `elfi-parser` crate is responsible for converting the `.elf` text format into an `automerge` document instance.

### 2.1. Tree-sitter Grammar

A `grammar.js` will be created for `tree-sitter`. It will define the following structure:
-   A `source_file` consists of one or more `block` nodes.
-   A `block` node is composed of a `metadata_section` and a `content_section`.
-   The `metadata_section` will be recognized as text enclosed by `---`.
-   The `content_section` is the remaining text.

This approach allows for efficient and error-tolerant parsing of the file into a concrete syntax tree (CST).

### 2.2. CST to CRDT Conversion

The parser will provide a primary function:
`pub fn parse_to_doc(text: &str) -> Result<automerge::AutoCommit, ElfiError>`

The conversion process will be as follows:
1.  The input `text` is parsed by Tree-sitter into a CST.
2.  The parser iterates through the top-level `block` nodes of the CST.
3.  For each `block` node:
    a. The text content of its `metadata_section` child is extracted and parsed using a YAML parser (e.g., `serde_yaml`).
    b. A new map object is created within a new `automerge` transaction, representing the block.
    c. The parsed YAML fields (`id`, `type`, `metadata`) are inserted into the Automerge map.
    d. The `content` field is created as an `automerge::Text` object, initialized with the text from the `content_section` child node.
    e. This new block map is appended to a top-level `blocks` list in the Automerge document.
4.  The final, populated `automerge::AutoCommit` document is returned.
