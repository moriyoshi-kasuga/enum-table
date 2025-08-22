use crate::{EnumTable, Enumable};

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

impl<'de, K, V, const N: usize> serde::Deserialize<'de> for EnumTable<K, V, N>
where
    K: Enumable + serde::Deserialize<'de> + core::fmt::Debug,
    V: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::marker::PhantomData;
        use serde::de::{MapAccess, Visitor};

        struct EnumTableVisitor<K, V, const N: usize> {
            _phantom: PhantomData<(K, V)>,
        }

        impl<'de, K, V, const N: usize> Visitor<'de> for EnumTableVisitor<K, V, N>
        where
            K: Enumable + serde::Deserialize<'de> + core::fmt::Debug,
            V: serde::Deserialize<'de>,
        {
            type Value = EnumTable<K, V, N>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a map with all enum variants as keys")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                use crate::EnumTableFromVecError;

                let mut values: Vec<(K, V)> = Vec::with_capacity(N);

                while let Some((key, value)) = map.next_entry::<K, V>()? {
                    values.push((key, value));
                }

                match EnumTable::try_from_vec(values) {
                    Ok(t) => Ok(t),
                    Err(EnumTableFromVecError::InvalidSize { expected, found }) => {
                        Err(serde::de::Error::invalid_length(
                            found,
                            &format!("expected {expected} entries, found {found}").as_str(),
                        ))
                    }
                    Err(EnumTableFromVecError::MissingVariant(variant)) => {
                        Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Str(&format!("{variant:?}")),
                            &"all enum variants must be present",
                        ))
                    }
                }
            }
        }

        deserializer.deserialize_map(EnumTableVisitor::<K, V, N> {
            _phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, Enumable, serde::Serialize, serde::Deserialize)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    const TABLES: EnumTable<Color, &'static str, { Color::COUNT }> =
        crate::et!(Color, &'static str, |color| match color {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
        });

    #[test]
    fn serde_serialize() {
        let json = serde_json::to_string(&TABLES).unwrap();
        assert!(json.contains(r#""Red":"Red""#));
        assert!(json.contains(r#""Green":"Green""#));
        assert!(json.contains(r#""Blue":"Blue""#));
    }

    #[test]
    fn serde_deserialize() {
        let json = r#"{"Red":"Red","Green":"Green","Blue":"Blue"}"#;
        let table: EnumTable<Color, &str, { Color::COUNT }> = serde_json::from_str(json).unwrap();

        assert_eq!(table.get(&Color::Red), &"Red");
        assert_eq!(table.get(&Color::Green), &"Green");
        assert_eq!(table.get(&Color::Blue), &"Blue");
    }

    #[test]
    fn serde_roundtrip() {
        let original = TABLES;
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: EnumTable<Color, &str, { Color::COUNT }> =
            serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn serde_missing_variant_error() {
        // Missing Blue variant
        let json = r#"{"Red":"Red","Green":"Green"}"#;
        let result: Result<EnumTable<Color, &str, { Color::COUNT }>, _> =
            serde_json::from_str(json);

        assert!(result.is_err());
    }
}
