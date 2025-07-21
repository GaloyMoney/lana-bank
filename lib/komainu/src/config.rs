use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct KomainuEndpoints {
    #[serde(with = "url_serde")]
    pub production_url: Url,
    #[serde(with = "url_serde")]
    pub test_url: Url,
}

impl Default for KomainuEndpoints {
    fn default() -> Self {
        Self {
            production_url: "https://api.komainu.io".parse().expect("valid URL"),
            test_url: "https://api-demo.komainu.io".parse().expect("valid URL"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KomainuConfig {
    pub api_user: String,
    pub api_secret: String,
    pub secret_key: KomainuSecretKey,
    pub komainu_test: bool,
    pub webhook_secret: Vec<u8>,
    pub endpoints: KomainuEndpoints,
}

impl KomainuConfig {
    pub const fn url(&self) -> &Url {
        if self.komainu_test {
            &self.endpoints.test_url
        } else {
            &self.endpoints.production_url
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum KomainuSecretKey {
    Encrypted { dem: String, passphrase: String },
    Plain { dem: String },
}

mod url_serde {
    use reqwest::Url;
    use serde::{
        de::{Error, Visitor},
        Deserializer, Serializer,
    };

    pub fn serialize<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(url.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UrlVisitor;

        impl Visitor<'_> for UrlVisitor {
            type Value = Url;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing an URL")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Url::parse(s).map_err(|err| Error::custom(format!("{err}: {s:?}")))
            }
        }

        deserializer.deserialize_str(UrlVisitor)
    }
}
