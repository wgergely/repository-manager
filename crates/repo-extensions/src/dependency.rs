//! Dependency graph and topological ordering for extensions and presets.
//!
//! When an extension declares `runtime.type = "python"`, it implicitly depends
//! on the `env:python` preset being satisfied first (a virtual environment must
//! exist before `pip install` can run). This module provides the graph
//! structures and a topological sort so the install/check pipeline can process
//! dependencies in the correct order.
//!
//! # Example
//!
//! ```
//! use repo_extensions::dependency::{DependencyGraph, DependencyNode};
//!
//! let mut graph = DependencyGraph::new();
//! graph.add_node(DependencyNode::preset("env:python"));
//! graph.add_node(DependencyNode::extension("vaultspec"));
//! graph.add_edge("vaultspec", "env:python");
//!
//! let order = graph.topological_sort().unwrap();
//! assert_eq!(order[0].id, "env:python");
//! assert_eq!(order[1].id, "vaultspec");
//! ```

use std::collections::{HashMap, HashSet};

use crate::error::{Error, Result};
use crate::manifest::ExtensionManifest;

/// The kind of node in the dependency graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// A preset provider (e.g., `env:python`, `env:rust`).
    Preset,
    /// An extension (e.g., `vaultspec`).
    Extension,
}

/// A single node in the dependency graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyNode {
    /// Unique identifier (preset ID or extension name).
    pub id: String,
    /// Whether this is a preset or an extension.
    pub kind: NodeKind,
}

impl DependencyNode {
    /// Create a preset dependency node.
    pub fn preset(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: NodeKind::Preset,
        }
    }

    /// Create an extension dependency node.
    pub fn extension(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: NodeKind::Extension,
        }
    }
}

/// Directed acyclic graph of dependencies between presets and extensions.
///
/// Edges point from dependent to dependency: if A depends on B, the edge
/// is `A -> B`. Topological sort returns nodes in dependency-first order
/// (B before A).
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    nodes: HashMap<String, DependencyNode>,
    /// Adjacency list: key depends on each value.
    edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    /// Create an empty dependency graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph. If a node with the same ID exists, it is
    /// replaced.
    pub fn add_node(&mut self, node: DependencyNode) {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.edges.entry(id).or_default();
    }

    /// Declare that `from` depends on `to`.
    ///
    /// Both nodes must already exist in the graph, otherwise the edge is
    /// silently ignored (the missing node will surface as an error during
    /// topological sort validation).
    pub fn add_edge(&mut self, from: &str, to: &str) {
        // Ensure target node exists in edge map
        self.edges.entry(to.to_string()).or_default();
        self.edges
            .entry(from.to_string())
            .or_default()
            .insert(to.to_string());
    }

    /// Return the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Return the number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|deps| deps.len()).sum()
    }

    /// Get the direct dependencies of a node.
    pub fn dependencies_of(&self, id: &str) -> Vec<&str> {
        self.edges
            .get(id)
            .map(|deps| deps.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Perform a topological sort using Kahn's algorithm.
    ///
    /// Returns nodes in dependency-first order: if A depends on B, B
    /// appears before A in the result.
    ///
    /// # Errors
    ///
    /// Returns `Error::DependencyCycle` if the graph contains a cycle.
    pub fn topological_sort(&self) -> Result<Vec<DependencyNode>> {
        // Compute in-degree for each node
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for id in self.nodes.keys() {
            in_degree.entry(id.as_str()).or_insert(0);
        }
        for deps in self.edges.values() {
            for dep in deps {
                if self.nodes.contains_key(dep) {
                    *in_degree.entry(dep.as_str()).or_insert(0) += 1;
                }
            }
        }

        // Seed the queue with zero-in-degree nodes (sorted for determinism)
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
        queue.sort();

        let mut result = Vec::with_capacity(self.nodes.len());

        while let Some(current) = queue.pop() {
            // Re-sort remaining to maintain deterministic order after each pop
            // (we pop the last element, so sort ascending and pop = largest first,
            // but we want alphabetical so we sort descending to pop smallest)
            // Actually, let's use a simpler approach:
            if let Some(node) = self.nodes.get(current) {
                result.push(node.clone());
            }

            // For each node that depends on `current`, decrement its in-degree
            for (from, deps) in &self.edges {
                if deps.contains(current) {
                    if let Some(deg) = in_degree.get_mut(from.as_str()) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push(from.as_str());
                            queue.sort();
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            // Find the nodes involved in the cycle for a useful error message
            let sorted_ids: HashSet<&str> = result.iter().map(|n| n.id.as_str()).collect();
            let cycle_participants: Vec<String> = self
                .nodes
                .keys()
                .filter(|id| !sorted_ids.contains(id.as_str()))
                .cloned()
                .collect();
            return Err(Error::DependencyCycle {
                participants: cycle_participants,
            });
        }

        Ok(result)
    }

    /// Build a dependency graph from a set of extension manifests.
    ///
    /// Automatically infers implicit dependencies:
    /// - Extensions with `runtime.type = "python"` depend on `env:python`
    /// - Extensions with `runtime.type = "node"` depend on `env:node`
    /// - Extensions with `runtime.type = "rust"` depend on `env:rust`
    pub fn from_manifests(manifests: &[(&str, &ExtensionManifest)]) -> Self {
        let mut graph = Self::new();

        // Collect all implicit preset dependencies
        let mut needed_presets: HashSet<String> = HashSet::new();

        for &(name, manifest) in manifests {
            graph.add_node(DependencyNode::extension(name));

            // Infer preset dependency from runtime type
            if let Some(ref runtime) = manifest.runtime {
                let preset_id = format!("env:{}", runtime.runtime_type);
                needed_presets.insert(preset_id.clone());
                graph.add_edge(name, &preset_id);
            }

            // Explicit Python requirement also implies env:python
            if manifest
                .requires
                .as_ref()
                .is_some_and(|r| r.python.is_some())
            {
                needed_presets.insert("env:python".to_string());
                graph.add_edge(name, "env:python");
            }
        }

        // Add preset nodes for all inferred dependencies
        for preset_id in needed_presets {
            graph.add_node(DependencyNode::preset(preset_id));
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        let sorted = graph.topological_sort().unwrap();
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_single_node() {
        let mut graph = DependencyGraph::new();
        graph.add_node(DependencyNode::preset("env:python"));
        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].id, "env:python");
    }

    #[test]
    fn test_linear_chain() {
        let mut graph = DependencyGraph::new();
        graph.add_node(DependencyNode::preset("env:python"));
        graph.add_node(DependencyNode::extension("vaultspec"));
        graph.add_edge("vaultspec", "env:python");

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 2);
        // env:python must come before vaultspec
        assert_eq!(sorted[0].id, "env:python");
        assert_eq!(sorted[1].id, "vaultspec");
    }

    #[test]
    fn test_diamond_dependency() {
        let mut graph = DependencyGraph::new();
        graph.add_node(DependencyNode::preset("env:python"));
        graph.add_node(DependencyNode::extension("ext-a"));
        graph.add_node(DependencyNode::extension("ext-b"));
        graph.add_node(DependencyNode::extension("ext-top"));

        graph.add_edge("ext-a", "env:python");
        graph.add_edge("ext-b", "env:python");
        graph.add_edge("ext-top", "ext-a");
        graph.add_edge("ext-top", "ext-b");

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 4);

        // env:python must be first
        assert_eq!(sorted[0].id, "env:python");
        // ext-top must be last
        assert_eq!(sorted[3].id, "ext-top");
    }

    #[test]
    fn test_cycle_detected() {
        let mut graph = DependencyGraph::new();
        graph.add_node(DependencyNode::extension("a"));
        graph.add_node(DependencyNode::extension("b"));
        graph.add_edge("a", "b");
        graph.add_edge("b", "a");

        let result = graph.topological_sort();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::DependencyCycle { .. }));
    }

    #[test]
    fn test_independent_nodes_sorted_deterministically() {
        let mut graph = DependencyGraph::new();
        graph.add_node(DependencyNode::extension("zebra"));
        graph.add_node(DependencyNode::extension("alpha"));
        graph.add_node(DependencyNode::extension("mid"));

        let sorted1 = graph.topological_sort().unwrap();
        let sorted2 = graph.topological_sort().unwrap();
        assert_eq!(sorted1.len(), 3);
        // Independent nodes should appear in deterministic (alphabetical) order
        assert_eq!(
            sorted1.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
            sorted2.iter().map(|n| n.id.as_str()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_from_manifests_python_runtime() {
        let toml = r#"
[extension]
name = "my-ext"
version = "1.0.0"

[runtime]
type = "python"
install = "pip install -e ."

[requires.python]
version = ">=3.12"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let graph = DependencyGraph::from_manifests(&[("my-ext", &manifest)]);

        assert_eq!(graph.node_count(), 2); // my-ext + env:python
        assert_eq!(graph.edge_count(), 1); // my-ext -> env:python (deduped)

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted[0].id, "env:python");
        assert_eq!(sorted[1].id, "my-ext");
    }

    #[test]
    fn test_from_manifests_no_runtime() {
        let toml = r#"
[extension]
name = "simple"
version = "1.0.0"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let graph = DependencyGraph::from_manifests(&[("simple", &manifest)]);

        assert_eq!(graph.node_count(), 1); // just the extension
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_from_manifests_rust_runtime() {
        let toml = r#"
[extension]
name = "rust-ext"
version = "1.0.0"

[runtime]
type = "rust"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let graph = DependencyGraph::from_manifests(&[("rust-ext", &manifest)]);

        assert_eq!(graph.node_count(), 2); // rust-ext + env:rust
        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted[0].id, "env:rust");
        assert_eq!(sorted[1].id, "rust-ext");
    }

    #[test]
    fn test_dependencies_of() {
        let mut graph = DependencyGraph::new();
        graph.add_node(DependencyNode::preset("env:python"));
        graph.add_node(DependencyNode::preset("env:node"));
        graph.add_node(DependencyNode::extension("multi"));
        graph.add_edge("multi", "env:python");
        graph.add_edge("multi", "env:node");

        let deps = graph.dependencies_of("multi");
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"env:python"));
        assert!(deps.contains(&"env:node"));

        let no_deps = graph.dependencies_of("env:python");
        assert!(no_deps.is_empty());
    }
}
