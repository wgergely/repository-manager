use repo_fs::NormalizedPath;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_normalization_invariants(s in "\\PC*") {
        // "s" will be a random string.
        let path = NormalizedPath::new(&s);
        let as_str = path.as_str();

        // Invariant 1: No backslashes in normalized path
        prop_assert!(!as_str.contains('\\'));

        // Invariant 2: No double slashes (unless network path start, which we might want to verify)
        // Our cleaner implementation preserves // at start for network paths, but collapses others.
        // Let's check for "///" which should definitely be collapsed if we assume "clean" works a certain way?
        // Actually, NormalizedPath logic says: "starts_with // && !starts_with ///" is network.
        // And inside loop it skips empty components.
        // So internal double slashes "a//b" should become "a/b".
        
        // Check for double slashes
        // We allow starting with // (network), but not ///
        let is_network = as_str.starts_with("//") && !as_str.starts_with("///");
        
        if is_network {
             // Skip the first 2 bytes (safe because we strictly matched "//")
             let remainder = &as_str[2..];
             prop_assert!(!remainder.contains("//"));
        } else {
             // Not a network path, should not contain // anywhere
             prop_assert!(!as_str.contains("//"));
        }

        // Invariant 3: to_native() roundtrip (mostly)
        // Note: converting back to native on windows puts backslash back.
        // converting *that* back to NormalizedPath should be identity equal to first result?
        // Let's try: NormalizedPath(s) -> native -> struct -> NormalizedPath == NormalizedPath(s)
        
        let native = path.to_native();
        let roundtripped = NormalizedPath::new(native);
        prop_assert_eq!(path, roundtripped);
    }

    #[test]
    fn test_join_properties(a in "\\PC*", b in "\\PC*") {
        let p1 = NormalizedPath::new(&a);
        let joined = p1.join(&b);
        
        // Joined path should start with p1 (unless p1 was relative and "popable" by b's ..s, 
        // OR unless b is absolute?
        // Our join implementation:
        // if self.inner.ends_with('/') ... format!("{}{}", ...)
        // else format!("{}/{}", ...)
        // THEN clean().
        
        // If b is NOT absolute and doesn't start with .., it should likely start with p1's text (if p1 didn't track back).
        // This is complex to assert generally.
        
        // Simple assertion: result is normalized
        prop_assert!(!joined.as_str().contains('\\'));
        
        // Assertion: if b is empty, result equals a (normalized)
        if b.is_empty() || b == "." {
             prop_assert_eq!(joined, p1);
        }
    }
}
