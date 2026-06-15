//! Typed identifiers using UUIDv7 for temporal ordering.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use uuid::Uuid;

/// A typed, ordered identifier.
///
/// Uses UUIDv7 internally so IDs are monotonically increasing
/// and sortable by creation time. The phantom type `T` prevents
/// mixing IDs across domain boundaries.
///
/// All trait impls ignore `T` — only the inner UUID matters.
pub struct Id<T> {
    value: Uuid,
    _phantom: PhantomData<fn() -> T>,
}

// ── Manual impls that do NOT require bounds on T ────────────────

impl<T> Id<T> {
    /// Generate a new temporally-ordered identifier.
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: Uuid::now_v7(),
            _phantom: PhantomData,
        }
    }

    /// The raw UUID.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.value
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Uuid::deserialize(deserializer)?;
        Ok(Self {
            value,
            _phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntity;

    #[test]
    fn ids_are_monotonic() {
        let a = Id::<TestEntity>::new();
        let b = Id::<TestEntity>::new();
        assert!(b > a);
    }

    #[test]
    fn roundtrip_json() {
        let id = Id::<TestEntity>::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: Id<TestEntity> = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn copy_semantics() {
        let a = Id::<TestEntity>::new();
        let b = a; // copy
        assert_eq!(a, b); // a is still usable
    }
}
