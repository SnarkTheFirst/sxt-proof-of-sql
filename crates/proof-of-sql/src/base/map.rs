/// Alias for the `IndexMap` with a hash builder of type `BuildHasherDefault<AHasher>`
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, core::hash::BuildHasherDefault<ahash::AHasher>>;
pub(crate) type IndexSet<T> = indexmap::IndexSet<T, core::hash::BuildHasherDefault<ahash::AHasher>>;

/// Create an [`IndexMap`][self::IndexMap] from a list of key-value pairs
#[cfg(test)]
macro_rules! indexmap {
    ($($key:expr => $value:expr,)+) => { indexmap::indexmap_with_default!{ahash::AHasher; $($key => $value),+} };
    ($($key:expr => $value:expr),*) => { indexmap::indexmap_with_default!{ahash::AHasher; $($key => $value),*} };
}

/// Create an [`IndexSet`][self::IndexSet] from a list of values
macro_rules! indexset {
    ($($value:expr,)+) => { indexmap::indexset_with_default!{ahash::AHasher; $($value),+} };
    ($($value:expr),*) => { indexmap::indexset_with_default!{ahash::AHasher; $($value),*} };
}

#[cfg(test)]
pub(crate) use indexmap;
pub(crate) use indexset;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn indexmap_macro_preserves_insertion_order_and_values() {
        let map = indexmap! {
            "first" => 10,
            "second" => 20,
            "third" => 30,
        };

        let entries = map
            .iter()
            .map(|(key, value)| (*key, *value))
            .collect::<Vec<_>>();
        assert_eq!(entries, [("first", 10), ("second", 20), ("third", 30)]);
    }

    #[test]
    fn indexmap_macro_accepts_empty_input() {
        let map: IndexMap<&str, i32> = indexmap! {};

        assert!(map.is_empty());
        assert_eq!(map.capacity(), 0);
    }

    #[test]
    fn indexset_macro_preserves_first_insertion_order_and_deduplicates() {
        let set = indexset!["north", "east", "north", "west", "east"];

        let values = set.iter().copied().collect::<Vec<_>>();
        assert_eq!(values, ["north", "east", "west"]);
    }

    #[test]
    fn indexset_macro_supports_trailing_comma_form() {
        let set = indexset![3, 1, 4, 1, 5,];

        let values = set.iter().copied().collect::<Vec<_>>();
        assert_eq!(values, [3, 1, 4, 5]);
    }
}
