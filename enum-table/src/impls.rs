use std::ops::{Index, IndexMut};

use crate::{EnumTable, Enumable};

impl<K: Enumable + core::fmt::Debug, V: core::fmt::Debug, const N: usize> core::fmt::Debug
    for EnumTable<K, V, N>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Enumable, V: Clone, const N: usize> Clone for EnumTable<K, V, N> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<K: Enumable, V: PartialEq, const N: usize> PartialEq for EnumTable<K, V, N> {
    fn eq(&self, other: &Self) -> bool {
        self.table.eq(&other.table)
    }
}

impl<K: Enumable, V: Eq, const N: usize> Eq for EnumTable<K, V, N> {}

impl<K: Enumable, V: std::hash::Hash, const N: usize> std::hash::Hash for EnumTable<K, V, N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.table.hash(state);
    }
}

impl<K: Enumable, V: Default, const N: usize> Default for EnumTable<K, V, N> {
    fn default() -> Self {
        EnumTable::new_with_fn(|_| Default::default())
    }
}

impl<K: Enumable, V, const N: usize> Index<K> for EnumTable<K, V, N> {
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(&index)
    }
}

impl<K: Enumable, V, const N: usize> IndexMut<K> for EnumTable<K, V, N> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(&index)
    }
}

#[cfg(feature = "serde")]
impl<K, V, const N: usize> serde::Serialize for EnumTable<K, V, N>
where
    K: Enumable + serde::Serialize,
    V: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(N))?;
        for (key, value) in self.iter() {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V, const N: usize> serde::Deserialize<'de> for EnumTable<K, V, N>
where
    K: Enumable + serde::Deserialize<'de> + Eq + std::hash::Hash,
    V: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::collections::HashMap;
        use std::fmt;
        use std::marker::PhantomData;

        struct EnumTableVisitor<K, V, const N: usize> {
            _phantom: PhantomData<(K, V)>,
        }

        impl<'de, K, V, const N: usize> Visitor<'de> for EnumTableVisitor<K, V, N>
        where
            K: Enumable + serde::Deserialize<'de> + Eq + std::hash::Hash,
            V: serde::Deserialize<'de>,
        {
            type Value = EnumTable<K, V, N>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with all enum variants as keys")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values: HashMap<K, V> = HashMap::new();

                while let Some((key, value)) = map.next_entry::<K, V>()? {
                    values.insert(key, value);
                }

                // Ensure all variants are present
                for variant in K::VARIANTS {
                    if !values.contains_key(variant) {
                        return Err(serde::de::Error::missing_field("enum variant"));
                    }
                }

                // Build the EnumTable
                let mut builder = crate::builder::EnumTableBuilder::<K, V, N>::new();
                for variant in K::VARIANTS {
                    if let Some(value) = values.remove(variant) {
                        builder.push(variant, value);
                    }
                }

                Ok(builder.build_to())
            }
        }

        deserializer.deserialize_map(EnumTableVisitor::<K, V, N> {
            _phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{Hash, Hasher};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    enum Color {
        Red,
        Green,
        Blue,
    }

    impl Enumable for Color {
        const VARIANTS: &'static [Self] = &[Color::Red, Color::Green, Color::Blue];
    }

    const TABLES: EnumTable<Color, &'static str, { Color::COUNT }> =
        crate::et!(Color, &'static str, |color| match color {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
        });

    const ANOTHER_TABLES: EnumTable<Color, &'static str, { Color::COUNT }> =
        crate::et!(Color, &'static str, |color| match color {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
        });

    #[test]
    fn debug_impl() {
        assert_eq!(
            format!("{:?}", TABLES),
            r#"{Red: "Red", Green: "Green", Blue: "Blue"}"#
        );
    }

    #[test]
    fn clone_impl() {
        let cloned = TABLES.clone();
        assert_eq!(cloned, TABLES);
    }

    #[test]
    fn eq_impl() {
        assert!(TABLES == ANOTHER_TABLES);
        assert!(TABLES != EnumTable::new_with_fn(|_| "Unknown"));
    }

    #[test]
    fn hash_impl() {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        TABLES.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        ANOTHER_TABLES.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn default_impl() {
        let default_table: EnumTable<Color, &'static str, { Color::COUNT }> = EnumTable::default();
        assert_eq!(default_table.get(&Color::Red), &"");
        assert_eq!(default_table.get(&Color::Green), &"");
        assert_eq!(default_table.get(&Color::Blue), &"");
    }

    #[test]
    fn index_impl() {
        assert_eq!(TABLES[Color::Red], "Red");
        assert_eq!(TABLES[Color::Green], "Green");
        assert_eq!(TABLES[Color::Blue], "Blue");

        let mut mutable_table = TABLES.clone();
        mutable_table[Color::Red] = "Changed Red";
        assert_eq!(mutable_table[Color::Red], "Changed Red");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_serialize() {
        let json = serde_json::to_string(&TABLES).unwrap();
        assert!(json.contains(r#""Red":"Red""#));
        assert!(json.contains(r#""Green":"Green""#));
        assert!(json.contains(r#""Blue":"Blue""#));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_deserialize() {
        let json = r#"{"Red":"Red","Green":"Green","Blue":"Blue"}"#;
        let table: EnumTable<Color, &str, { Color::COUNT }> = serde_json::from_str(json).unwrap();

        assert_eq!(table.get(&Color::Red), &"Red");
        assert_eq!(table.get(&Color::Green), &"Green");
        assert_eq!(table.get(&Color::Blue), &"Blue");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip() {
        let original = TABLES;
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: EnumTable<Color, &str, { Color::COUNT }> =
            serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_missing_variant_error() {
        // Missing Blue variant
        let json = r#"{"Red":"Red","Green":"Green"}"#;
        let result: Result<EnumTable<Color, &str, { Color::COUNT }>, _> =
            serde_json::from_str(json);

        assert!(result.is_err());
    }
}
