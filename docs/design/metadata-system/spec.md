# Metadata System

**Goal**: Management of the `.repository` directory, acting as the state-of-truth for the repository manager.

## Responsibilities

- **Registry**: Storing the list of active tools and presets.
- **Schema Validation**: Ensuring `.repository/config.toml` (and others) are valid.
- **API**: Providing query access to other crates ("Is the python preset active?").
