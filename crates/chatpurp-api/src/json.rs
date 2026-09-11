//! Un même champ ne doit jamais avoir deux interprétations, même dans un extra ignoré.
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use std::fmt;

struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Strict;
        impl<'de> Visitor<'de> for Strict {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON sans champ dupliqué")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Unique, E> {
                Ok(Unique(Value::Bool(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Unique, E> {
                Ok(Unique(Value::Number(v.into())))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Unique, E> {
                Ok(Unique(Value::Number(v.into())))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Unique, E> {
                Number::from_f64(v)
                    .map(|n| Unique(Value::Number(n)))
                    .ok_or_else(|| E::custom("nombre invalide"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Unique, E> {
                Ok(Unique(Value::String(v.into())))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Unique, E> {
                Ok(Unique(Value::String(v)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Unique, A::Error> {
                let mut result = Vec::new();
                while let Some(Unique(value)) = a.next_element()? {
                    result.push(value);
                }
                Ok(Unique(Value::Array(result)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Unique, A::Error> {
                let mut result = Map::new();
                while let Some(key) = a.next_key::<String>()? {
                    if result.contains_key(&key) {
                        return Err(de::Error::custom("champ dupliqué"));
                    }
                    result.insert(key, a.next_value::<Unique>()?.0);
                }
                Ok(Unique(Value::Object(result)))
            }
        }
        d.deserialize_any(Strict)
    }
}
pub fn parse(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    serde_json::from_slice::<Unique>(bytes).map(|v| v.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_duplicates_and_trailing_bytes_are_rejected() {
        for input in [
            r#"{"identity_id":"oscar","identity_id":"alice"}"#,
            r#"{"extra":[{"a":1,"a":2}]}"#,
            "{}{}",
        ] {
            assert!(parse(input.as_bytes()).is_err());
        }
        assert!(parse(br#"{"extra":[null,true,1,-1,1.5,"ok"],"identity_id":"oscar"}"#).is_ok());
    }
}
