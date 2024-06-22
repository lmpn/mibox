use serde::{de::Visitor, Deserialize, Deserializer};

#[derive(Debug)]
pub struct BoxPath {
    path: std::path::PathBuf,
    base: String,
    name: String,
}

impl BoxPath {
    pub fn new<S: AsRef<str>>(path: S) -> Self {
        let components = path.as_ref().split("/").skip(1).collect::<Vec<_>>();
        let (base, name) = if components.len() == 0 {
            ("root".to_string(), "".to_string())
        } else if components.len() == 1 {
            ("root".to_string(), components[0].to_string())
        } else {
            (
                "root".to_string() + "/" + &components[0..components.len() - 1].join("/"),
                components[components.len() - 1].to_string(),
            )
        };
        let path: std::path::PathBuf = components.join("/").into();
        Self { path, base, name }
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn full_name(&self) -> String {
        if self.name.is_empty() {
            self.base.to_string()
        } else {
            format!("{}/{}", self.base, &self.name)
        }
    }
}

impl AsRef<std::path::Path> for BoxPath {
    fn as_ref(&self) -> &std::path::Path {
        &self.path
    }
}

struct BoxPathVisitor;
impl BoxPathVisitor {
    fn new() -> Self {
        BoxPathVisitor
    }
}
impl<'de> Visitor<'de> for BoxPathVisitor {
    // The type that our Visitor is going to produce.
    type Value = BoxPath;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a string with the following format: root or root/a/b/c.")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.starts_with("root/") || v.starts_with("root") {
            Ok(BoxPath::new(v))
        } else {
            Err(E::invalid_value(serde::de::Unexpected::Str(v), &self))
        }
    }
}

// This is the trait that informs Serde how to deserialize MyMap.
impl<'de> Deserialize<'de> for BoxPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_map(BoxPathVisitor::new())
    }
}
