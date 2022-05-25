///! A crate providing a simple data structure for caching id values.
///!
///! See the documentation for the [`IdCache<I, T>`] type for more information.
use id_collections::{Count, Id, IdVec};
use std::{borrow::Borrow, collections::HashMap, fmt::Debug, hash::Hash, ops::Index};

/// A cache which generates sequentially-assigned ids for unique values.
///
/// # Example
///
/// ```
/// use id_collections::id_type;
/// use id_cache::IdCache;
///
/// #[id_type]
/// struct WordId(u32);
///
/// let mut word_cache: IdCache<WordId, &str> = IdCache::new();
///
/// let foo_id = word_cache.make_id("foo");
/// let bar_id = word_cache.make_id("bar");
///
/// assert_eq!(word_cache[foo_id], "foo");
/// assert_eq!(word_cache[bar_id], "bar");
///
/// // ids for repeated values are reused:
/// assert_eq!(word_cache.make_id("foo"), foo_id);
/// ```
///
/// # Serde Support
///
/// When the `serde` Cargo feature is enabled, the `IdCache<I, T>` type can be serialized and
/// deserialized using [Serde](https://serde.rs). An `IdCache<I, T>` is serialized as a sequence
/// consisting of the unique values in the cache, ordered by id:
///
/// ```
/// # #[cfg(feature = "serde")]
/// # {
/// use id_collections::id_type;
/// use id_cache::IdCache;
///
/// #[id_type]
/// struct WordId(u32);
///
/// let mut word_cache: IdCache<WordId, &str> = IdCache::new();
/// word_cache.make_id("foo");
/// word_cache.make_id("bar");
///
/// let serialized = serde_json::to_string(&word_cache).unwrap();
/// assert_eq!(&serialized, r#"["foo","bar"]"#);
/// # }
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct IdCache<I: Id, T> {
    #[cfg_attr(feature = "serde", serde(bound(serialize = "T: serde::Serialize")))]
    id_to_value: IdVec<I, T>,
    #[cfg_attr(feature = "serde", serde(skip))]
    value_to_id: HashMap<T, I>,
}

#[cfg(feature = "serde")]
impl<'de, I: Id, T: Eq + Hash + Clone + serde::Deserialize<'de>> serde::Deserialize<'de>
    for IdCache<I, T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id_to_value = IdVec::<I, T>::deserialize(deserializer)?;

        let mut value_to_id = HashMap::new();
        for (id, value) in &id_to_value {
            let existing = value_to_id.insert(value.clone(), id);
            if existing.is_some() {
                use serde::de::Error;
                return Err(D::Error::custom("duplicate value in IdCache"));
            }
        }

        Ok(IdCache {
            id_to_value,
            value_to_id,
        })
    }
}

#[cfg(feature = "serde")]
mod serde_test {
    #[test]
    fn test_round_trip() {
        use crate::IdCache;
        use id_collections::id_type;

        #[id_type]
        struct TestId(u32);

        let mut cache: IdCache<u32, String> = IdCache::new();
        cache.make_id("foo".to_owned());
        cache.make_id("bar".to_owned());

        let serialized = serde_json::to_string(&cache).unwrap();
        assert_eq!(serialized, r#"["foo","bar"]"#);

        let deserialized = serde_json::from_str::<IdCache<u32, String>>(&serialized).unwrap();
        assert_eq!(&deserialized, &cache);
    }

    #[test]
    fn test_duplicate_err() {
        use crate::IdCache;

        let result = serde_json::from_str::<IdCache<u32, String>>(r#"["foo","foo"]"#);
        assert!(result.is_err());
    }
}

impl<I: Id, T> Default for IdCache<I, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Id + Debug, T: Debug> Debug for IdCache<I, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id_to_value.fmt(f)
    }
}

impl<I: Id, T: PartialEq> PartialEq for IdCache<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.id_to_value == other.id_to_value
    }
}

impl<I: Id, T: Eq> Eq for IdCache<I, T> {}

impl<I: Id, T> IdCache<I, T> {
    /// Constructs a new, empty `IdCache<I, T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use id_cache::IdCache;
    /// let cache: IdCache<u32, &str> = IdCache::new();
    /// assert!(cache.is_empty());
    /// ```
    pub fn new() -> Self {
        IdCache {
            id_to_value: IdVec::new(),
            value_to_id: HashMap::new(),
        }
    }

    /// Constructs a new, empty `IdCache<I, T>` with space to hold at least `capacity` unique
    /// values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use id_cache::IdCache;
    /// let mut cache: IdCache<u32, &str> = IdCache::with_capacity(100);
    /// assert!(cache.is_empty());
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        IdCache {
            id_to_value: IdVec::with_capacity(capacity),
            value_to_id: HashMap::with_capacity(capacity),
        }
    }

    /// Returns the total number of ids that have been assigned to unique values in the
    /// `IdCache<I, T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use id_cache::IdCache;
    /// let mut cache: IdCache<u32, &str> = IdCache::new();
    /// assert!(cache.count().is_empty());
    /// cache.make_id("foo");
    /// cache.make_id("bar");
    /// assert_eq!(cache.count().to_value(), 2);
    /// cache.make_id("foo"); // value already present, so does not assign a new id
    /// assert_eq!(cache.count().to_value(), 2);
    /// ```
    pub fn count(&self) -> Count<I> {
        self.id_to_value.count()
    }

    /// Returns the total number of unique values in the `IdCache<I, T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use id_cache::IdCache;
    /// let mut cache: IdCache<u32, &str> = IdCache::new();
    /// assert_eq!(cache.len(), 0);
    /// cache.make_id("foo");
    /// cache.make_id("bar");
    /// assert_eq!(cache.len(), 2);
    /// cache.make_id("foo"); // value already present, so does not increase the len
    /// assert_eq!(cache.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.id_to_value.len()
    }

    /// Returns `true` if the `IdCache<I, T>` contains no values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use id_cache::IdCache;
    /// let mut cache: IdCache<u32, &str> = IdCache::new();
    /// assert!(cache.is_empty());
    /// cache.make_id("foo");
    /// assert!(!cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.id_to_value.is_empty()
    }

    /// Ensures `value` has an id in the `IdCache<I, T>`, and returns that id.
    ///
    /// If `value` is already present in the `IdCache<I, T>`, then `make_id` returns its existing
    /// id. Otherwise, `make_id` returns a new sequentally-assigned id.
    ///
    /// # Panics
    ///
    /// Panics if the number of ids in the `IdCache<I, T>` overflows `I`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use id_cache::IdCache;
    /// let mut cache: IdCache<u32, &str> = IdCache::new();
    /// assert_eq!(cache.make_id("foo"), 0);
    /// assert_eq!(cache.make_id("bar"), 1);
    /// assert_eq!(cache.make_id("foo"), 0);
    /// ```
    pub fn make_id(&mut self, value: T) -> I
    where
        T: Eq + Hash + Clone,
    {
        *self
            .value_to_id
            .entry(value)
            .or_insert_with_key(|value| self.id_to_value.push(value.clone()))
    }

    /// Returns the id of a value in the `IdCache<I, T>`, or `None` if the value is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// # use id_cache::IdCache;
    /// let mut cache: IdCache<u32, &str> = IdCache::new();
    /// let foo_id = cache.make_id("foo");
    /// assert_eq!(cache.get_id(&"foo"), Some(foo_id));
    /// assert_eq!(cache.get_id(&"bar"), None);
    /// ```
    pub fn get_id<U>(&self, value: &U) -> Option<I>
    where
        T: Borrow<U> + Eq + Hash,
        U: Eq + Hash,
    {
        self.value_to_id.get(value).cloned()
    }

    /// Returns a reference to the value in the `IdCache<I, T>` associated with a given `id`, or
    /// `None` if the id has not been assigned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use id_cache::IdCache;
    /// let mut cache: IdCache<u32, &str> = IdCache::new();
    /// let foo_id = cache.make_id("foo");
    /// assert_eq!(foo_id, 0);
    /// assert_eq!(cache.get_value(foo_id), Some(&"foo"));
    /// assert_eq!(cache.get_value(1), None);
    /// ```
    pub fn get_value(&self, id: I) -> Option<&T> {
        self.id_to_value.get(id)
    }
}

impl<I: Id, T, J: Borrow<I>> Index<J> for IdCache<I, T> {
    type Output = T;

    /// Returns a reference to the value in the `IdCache<I, T>` associated with a given `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` has not been assigned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use id_cache::IdCache;
    /// let mut cache: IdCache<u32, &str> = IdCache::new();
    /// let foo_id = cache.make_id("foo");
    /// assert_eq!(cache[foo_id], "foo");
    /// ```
    #[inline]
    fn index(&self, id: J) -> &Self::Output {
        let id = *id.borrow();
        &self.id_to_value[id]
    }
}
