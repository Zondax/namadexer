use serde::{Deserialize, Deserializer};

pub(crate) fn serialize_hex<S>(v: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&hex::encode(v.as_slice()))
}

pub(crate) fn from_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| hex::decode(string).map_err(|err| Error::custom(err.to_string())))
}

pub(crate) fn serialize_optional_hex<S>(
    v: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(value) = v {
        serializer.serialize_str(&hex::encode(value.as_slice()))
    } else {
        serializer.serialize_str("")
    }
}
