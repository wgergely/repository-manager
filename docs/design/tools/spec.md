# Tools Subsystem

**Goal**: Define the "Tools" crate, which handles the definition, discovery, and registration of external tools like Claue Code, IDEs, and CLI utilities.

## Responsibilities

- **Discovery**: Identifying tools via `.agent` folders, `.vscode` folders, or other markers.
- **Schema**: Defining the `Tool` trait/struct.
- **Registration**: informing the Metadata System of available tools.
