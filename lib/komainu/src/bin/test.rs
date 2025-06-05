use komainu::{KomainuClient, KomainuConfig, KomainuProxy, KomainuSecretKey};

#[tokio::main]
async fn main() {
    let config = KomainuConfig {
        api_user: "".to_string(),
        api_secret: "".to_string(),
        secret_key: KomainuSecretKey::Encrypted {
            passphrase: "".to_string(),
            dem: r#"-----BEGIN ENCRYPTED PRIVATE KEY-----

-----END ENCRYPTED PRIVATE KEY-----"#
                .to_string(),
        },
        proxy: Some(KomainuProxy::Socks5("localhost:9915".to_string())),
        komainu_test: true,
    };

    let client = KomainuClient::new(config);

    let wallets = client.list_wallets().await;

    println!("{:#?}", wallets);
}
