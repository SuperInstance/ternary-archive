#![forbid(unsafe_code)]

//! Persistent storage and retrieval of ternary knowledge in balanced ternary {-1, 0, +1} systems.
//!
//! This crate provides structures for archiving, indexing, and retrieving ternary knowledge
//! with conservation law verification and lifecycle management.

use std::collections::HashMap;

/// A ternary value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Ternary {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

/// An immutable record in the archive. Once written, never modified.
#[derive(Debug, Clone)]
pub struct Scroll {
    id: u64,
    category: String,
    key: String,
    value: Ternary,
    metadata: HashMap<String, String>,
    timestamp: u64,
}

impl Scroll {
    pub fn new(id: u64, category: &str, key: &str, value: Ternary, timestamp: u64) -> Self {
        Self {
            id,
            category: category.to_string(),
            key: key.to_string(),
            value,
            metadata: HashMap::new(),
            timestamp,
        }
    }

    pub fn with_metadata(mut self, k: &str, v: &str) -> Self {
        self.metadata.insert(k.to_string(), v.to_string());
        self
    }

    pub fn id(&self) -> u64 { self.id }
    pub fn category(&self) -> &str { &self.category }
    pub fn key(&self) -> &str { &self.key }
    pub fn value(&self) -> Ternary { self.value }
    pub fn timestamp(&self) -> u64 { self.timestamp }
    pub fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
}

/// Fast lookup index over the archive.
#[derive(Debug, Clone)]
pub struct Index {
    by_category: HashMap<String, Vec<u64>>,
    by_key: HashMap<String, Vec<u64>>,
    by_value: HashMap<Ternary, Vec<u64>>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            by_category: HashMap::new(),
            by_key: HashMap::new(),
            by_value: HashMap::new(),
        }
    }

    pub fn insert(&mut self, scroll: &Scroll) {
        self.by_category
            .entry(scroll.category.clone())
            .or_default()
            .push(scroll.id);
        self.by_key
            .entry(scroll.key.clone())
            .or_default()
            .push(scroll.id);
        self.by_value
            .entry(scroll.value)
            .or_default()
            .push(scroll.id);
    }

    pub fn lookup_category(&self, cat: &str) -> &[u64] {
        self.by_category.get(cat).map_or(&[], |v| v.as_slice())
    }

    pub fn lookup_key(&self, key: &str) -> &[u64] {
        self.by_key.get(key).map_or(&[], |v| v.as_slice())
    }

    pub fn lookup_value(&self, val: Ternary) -> &[u64] {
        self.by_value.get(&val).map_or(&[], |v| v.as_slice())
    }

    pub fn category_count(&self) -> usize {
        self.by_category.len()
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

/// Browse scrolls by category.
#[derive(Debug, Clone)]
pub struct Catalog {
    categories: HashMap<String, Vec<Scroll>>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
        }
    }

    pub fn add(&mut self, scroll: Scroll) {
        self.categories
            .entry(scroll.category().to_string())
            .or_default()
            .push(scroll);
    }

    pub fn browse(&self, category: &str) -> &[Scroll] {
        self.categories.get(category).map_or(&[], |v| v.as_slice())
    }

    pub fn categories(&self) -> Vec<&str> {
        self.categories.keys().map(|s| s.as_str()).collect()
    }

    pub fn total_scrolls(&self) -> usize {
        self.categories.values().map(|v| v.len()).sum()
    }

    pub fn category_size(&self, cat: &str) -> usize {
        self.categories.get(cat).map_or(0, |v| v.len())
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify that archive contents don't violate ternary conservation laws.
/// In balanced ternary systems, the sum of all values should tend toward zero.
#[derive(Debug, Clone)]
pub struct Conservation {
    total_sum: i64,
    count: u64,
}

impl Conservation {
    pub fn new() -> Self {
        Self { total_sum: 0, count: 0 }
    }

    pub fn record(&mut self, value: Ternary) {
        self.total_sum += value.to_i8() as i64;
        self.count += 1;
    }

    /// Current balance: sum of all recorded values.
    pub fn balance(&self) -> i64 {
        self.total_sum
    }

    /// Whether the archive is balanced (sum == 0).
    pub fn is_balanced(&self) -> bool {
        self.total_sum == 0
    }

    /// Deviation from balance as a fraction of total count.
    pub fn deviation(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.total_sum as f64 / self.count as f64
    }

    /// Check if adding a value would violate a conservation threshold.
    pub fn would_violate(&self, value: Ternary, threshold: i64) -> bool {
        (self.total_sum + value.to_i8() as i64).unsigned_abs() as i64 > threshold
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

impl Default for Conservation {
    fn default() -> Self {
        Self::new()
    }
}

/// The main knowledge archive.
#[derive(Debug, Clone)]
pub struct Archive {
    scrolls: HashMap<u64, Scroll>,
    index: Index,
    catalog: Catalog,
    conservation: Conservation,
    next_id: u64,
}

impl Archive {
    pub fn new() -> Self {
        Self {
            scrolls: HashMap::new(),
            index: Index::new(),
            catalog: Catalog::new(),
            conservation: Conservation::new(),
            next_id: 1,
        }
    }

    /// Store a new scroll. Returns the scroll's ID.
    pub fn store(&mut self, category: &str, key: &str, value: Ternary, timestamp: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let scroll = Scroll::new(id, category, key, value, timestamp);
        self.index.insert(&scroll);
        self.conservation.record(value);
        self.catalog.add(scroll.clone());
        self.scrolls.insert(id, scroll);
        id
    }

    /// Retrieve a scroll by ID.
    pub fn retrieve(&self, id: u64) -> Option<&Scroll> {
        self.scrolls.get(&id)
    }

    /// Find scrolls by category.
    pub fn find_by_category(&self, cat: &str) -> Vec<&Scroll> {
        self.index.lookup_category(cat)
            .iter()
            .filter_map(|id| self.scrolls.get(id))
            .collect()
    }

    /// Find scrolls by key.
    pub fn find_by_key(&self, key: &str) -> Vec<&Scroll> {
        self.index.lookup_key(key)
            .iter()
            .filter_map(|id| self.scrolls.get(id))
            .collect()
    }

    /// Find scrolls by value.
    pub fn find_by_value(&self, val: Ternary) -> Vec<&Scroll> {
        self.index.lookup_value(val)
            .iter()
            .filter_map(|id| self.scrolls.get(id))
            .collect()
    }

    /// Total scrolls in archive.
    pub fn len(&self) -> usize {
        self.scrolls.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scrolls.is_empty()
    }

    /// Conservation balance.
    pub fn conservation_balance(&self) -> i64 {
        self.conservation.balance()
    }

    /// Is the archive conservation-balanced?
    pub fn is_balanced(&self) -> bool {
        self.conservation.is_balanced()
    }

    /// Browse the catalog.
    pub fn browse(&self, category: &str) -> &[Scroll] {
        self.catalog.browse(category)
    }

    /// List all categories.
    pub fn categories(&self) -> Vec<&str> {
        self.catalog.categories()
    }
}

impl Default for Archive {
    fn default() -> Self {
        Self::new()
    }
}

/// Lifecycle stage of archived knowledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleStage {
    Active,
    Deprecated,
    Archived,
    Expired,
}

/// Manages knowledge lifecycle: promote/demote scrolls through stages.
#[derive(Debug, Clone)]
pub struct ArchiveCurator {
    stages: HashMap<u64, LifecycleStage>,
    deprecation_reasons: HashMap<u64, String>,
}

impl ArchiveCurator {
    pub fn new() -> Self {
        Self {
            stages: HashMap::new(),
            deprecation_reasons: HashMap::new(),
        }
    }

    /// Register a scroll for lifecycle tracking.
    pub fn register(&mut self, id: u64) {
        self.stages.insert(id, LifecycleStage::Active);
    }

    /// Get the current stage.
    pub fn stage(&self, id: u64) -> Option<LifecycleStage> {
        self.stages.get(&id).copied()
    }

    /// Deprecate a scroll.
    pub fn deprecate(&mut self, id: u64, reason: &str) -> bool {
        if let Some(stage) = self.stages.get_mut(&id) {
            if *stage == LifecycleStage::Active {
                *stage = LifecycleStage::Deprecated;
                self.deprecation_reasons.insert(id, reason.to_string());
                return true;
            }
        }
        false
    }

    /// Archive a deprecated scroll.
    pub fn archive(&mut self, id: u64) -> bool {
        if let Some(stage) = self.stages.get_mut(&id) {
            if *stage == LifecycleStage::Deprecated {
                *stage = LifecycleStage::Archived;
                return true;
            }
        }
        false
    }

    /// Expire an archived scroll.
    pub fn expire(&mut self, id: u64) -> bool {
        if let Some(stage) = self.stages.get_mut(&id) {
            if *stage == LifecycleStage::Archived {
                *stage = LifecycleStage::Expired;
                return true;
            }
        }
        false
    }

    /// Count scrolls in each stage.
    pub fn count_by_stage(&self, stage: LifecycleStage) -> usize {
        self.stages.values().filter(|s| **s == stage).count()
    }

    /// Get deprecation reason.
    pub fn deprecation_reason(&self, id: u64) -> Option<&str> {
        self.deprecation_reasons.get(&id).map(|s| s.as_str())
    }

    /// Total tracked scrolls.
    pub fn total(&self) -> usize {
        self.stages.len()
    }
}

impl Default for ArchiveCurator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_roundtrip() {
        assert_eq!(Ternary::from_i8(Ternary::Neg.to_i8()), Some(Ternary::Neg));
        assert_eq!(Ternary::from_i8(Ternary::Zero.to_i8()), Some(Ternary::Zero));
        assert_eq!(Ternary::from_i8(Ternary::Pos.to_i8()), Some(Ternary::Pos));
    }

    #[test]
    fn test_scroll_creation() {
        let s = Scroll::new(1, "physics", "momentum", Ternary::Pos, 100);
        assert_eq!(s.id(), 1);
        assert_eq!(s.category(), "physics");
        assert_eq!(s.key(), "momentum");
        assert_eq!(s.value(), Ternary::Pos);
        assert_eq!(s.timestamp(), 100);
    }

    #[test]
    fn test_scroll_metadata() {
        let s = Scroll::new(1, "cat", "k", Ternary::Zero, 0)
            .with_metadata("author", "alice")
            .with_metadata("version", "1.0");
        assert_eq!(s.metadata().get("author"), Some(&"alice".to_string()));
        assert_eq!(s.metadata().get("version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_index_insert_and_lookup() {
        let mut idx = Index::new();
        let s = Scroll::new(1, "physics", "energy", Ternary::Neg, 0);
        idx.insert(&s);
        assert_eq!(idx.lookup_category("physics"), &[1]);
        assert_eq!(idx.lookup_key("energy"), &[1]);
        assert_eq!(idx.lookup_value(Ternary::Neg), &[1]);
        assert_eq!(idx.lookup_category("chemistry"), &[]);
    }

    #[test]
    fn test_index_category_count() {
        let mut idx = Index::new();
        idx.insert(&Scroll::new(1, "a", "x", Ternary::Zero, 0));
        idx.insert(&Scroll::new(2, "b", "y", Ternary::Pos, 0));
        assert_eq!(idx.category_count(), 2);
    }

    #[test]
    fn test_catalog_add_and_browse() {
        let mut cat = Catalog::new();
        cat.add(Scroll::new(1, "physics", "f1", Ternary::Pos, 0));
        cat.add(Scroll::new(2, "physics", "f2", Ternary::Neg, 0));
        cat.add(Scroll::new(3, "chemistry", "c1", Ternary::Zero, 0));
        assert_eq!(cat.browse("physics").len(), 2);
        assert_eq!(cat.browse("chemistry").len(), 1);
        assert_eq!(cat.browse("biology").len(), 0);
    }

    #[test]
    fn test_catalog_categories() {
        let mut cat = Catalog::new();
        cat.add(Scroll::new(1, "a", "x", Ternary::Zero, 0));
        cat.add(Scroll::new(2, "b", "y", Ternary::Zero, 0));
        let mut cats = cat.categories();
        cats.sort();
        assert_eq!(cats, vec!["a", "b"]);
    }

    #[test]
    fn test_catalog_total() {
        let mut cat = Catalog::new();
        assert_eq!(cat.total_scrolls(), 0);
        cat.add(Scroll::new(1, "a", "x", Ternary::Zero, 0));
        cat.add(Scroll::new(2, "a", "y", Ternary::Zero, 0));
        cat.add(Scroll::new(3, "b", "z", Ternary::Zero, 0));
        assert_eq!(cat.total_scrolls(), 3);
    }

    #[test]
    fn test_conservation_balanced() {
        let mut c = Conservation::new();
        c.record(Ternary::Pos);
        c.record(Ternary::Neg);
        assert!(c.is_balanced());
        assert_eq!(c.balance(), 0);
    }

    #[test]
    fn test_conservation_unbalanced() {
        let mut c = Conservation::new();
        c.record(Ternary::Pos);
        c.record(Ternary::Pos);
        assert!(!c.is_balanced());
        assert_eq!(c.balance(), 2);
    }

    #[test]
    fn test_conservation_deviation() {
        let mut c = Conservation::new();
        assert_eq!(c.deviation(), 0.0);
        c.record(Ternary::Pos);
        c.record(Ternary::Pos);
        assert_eq!(c.deviation(), 1.0);
    }

    #[test]
    fn test_conservation_would_violate() {
        let mut c = Conservation::new();
        c.record(Ternary::Pos);
        c.record(Ternary::Pos);
        assert!(c.would_violate(Ternary::Pos, 2)); // 3 > 2
        assert!(!c.would_violate(Ternary::Neg, 2)); // 1 <= 2
    }

    #[test]
    fn test_archive_store_and_retrieve() {
        let mut a = Archive::new();
        let id = a.store("physics", "energy", Ternary::Pos, 100);
        let scroll = a.retrieve(id).unwrap();
        assert_eq!(scroll.key(), "energy");
        assert_eq!(scroll.value(), Ternary::Pos);
    }

    #[test]
    fn test_archive_find_by_category() {
        let mut a = Archive::new();
        a.store("physics", "a", Ternary::Pos, 0);
        a.store("physics", "b", Ternary::Neg, 0);
        a.store("chemistry", "c", Ternary::Zero, 0);
        assert_eq!(a.find_by_category("physics").len(), 2);
    }

    #[test]
    fn test_archive_find_by_key() {
        let mut a = Archive::new();
        a.store("cat", "unique_key", Ternary::Zero, 0);
        assert_eq!(a.find_by_key("unique_key").len(), 1);
        assert_eq!(a.find_by_key("nonexistent").len(), 0);
    }

    #[test]
    fn test_archive_find_by_value() {
        let mut a = Archive::new();
        a.store("c", "a", Ternary::Pos, 0);
        a.store("c", "b", Ternary::Pos, 0);
        a.store("c", "c", Ternary::Neg, 0);
        assert_eq!(a.find_by_value(Ternary::Pos).len(), 2);
        assert_eq!(a.find_by_value(Ternary::Neg).len(), 1);
    }

    #[test]
    fn test_archive_len_and_empty() {
        let mut a = Archive::new();
        assert!(a.is_empty());
        a.store("c", "k", Ternary::Zero, 0);
        assert_eq!(a.len(), 1);
        assert!(!a.is_empty());
    }

    #[test]
    fn test_archive_conservation() {
        let mut a = Archive::new();
        a.store("c", "a", Ternary::Pos, 0);
        a.store("c", "b", Ternary::Neg, 0);
        assert!(a.is_balanced());
    }

    #[test]
    fn test_archive_browse() {
        let mut a = Archive::new();
        a.store("physics", "x", Ternary::Pos, 0);
        a.store("physics", "y", Ternary::Neg, 0);
        assert_eq!(a.browse("physics").len(), 2);
    }

    #[test]
    fn test_curator_lifecycle() {
        let mut cur = ArchiveCurator::new();
        cur.register(1);
        assert_eq!(cur.stage(1), Some(LifecycleStage::Active));
        cur.deprecate(1, "superseded");
        assert_eq!(cur.stage(1), Some(LifecycleStage::Deprecated));
        assert_eq!(cur.deprecation_reason(1), Some("superseded"));
        cur.archive(1);
        assert_eq!(cur.stage(1), Some(LifecycleStage::Archived));
        cur.expire(1);
        assert_eq!(cur.stage(1), Some(LifecycleStage::Expired));
    }

    #[test]
    fn test_curator_cannot_skip_stages() {
        let mut cur = ArchiveCurator::new();
        cur.register(1);
        assert!(!cur.expire(1)); // can't go Active -> Expired
        assert!(!cur.archive(1)); // can't go Active -> Archived
    }

    #[test]
    fn test_curator_count_by_stage() {
        let mut cur = ArchiveCurator::new();
        cur.register(1);
        cur.register(2);
        cur.deprecate(1, "old");
        assert_eq!(cur.count_by_stage(LifecycleStage::Active), 1);
        assert_eq!(cur.count_by_stage(LifecycleStage::Deprecated), 1);
    }
}
