use std::sync::Arc;

use deadpool_postgres::Pool;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ClientConfig, RootCertStore};
use tokio_postgres::NoTls;
use tracing::info;

use crate::settings::DatabaseSettings;

pub async fn create_pool(settings: &DatabaseSettings) -> anyhow::Result<Pool> {
  let pool = &settings.pool;

  let pool = if let Some(tls) = &settings.tls {
    let ca = CertificateDer::from_pem_file(&tls.ca_cert).unwrap();
    let mut root_store = RootCertStore::empty();
    root_store.add(ca).unwrap();

    let cert = CertificateDer::from_pem_file(&tls.client_cert).unwrap();
    let key = PrivateKeyDer::from_pem_file(&tls.client_key).unwrap();
    let client_config = ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
      .with_protocol_versions(rustls::DEFAULT_VERSIONS)
      .unwrap()
      .with_root_certificates(root_store.clone())
      .with_client_auth_cert(vec![cert], key)
      .unwrap();

    info!("using tls database connection");
    pool
      .create_pool(
        Some(deadpool_postgres::Runtime::Tokio1),
        tokio_postgres_rustls::MakeRustlsConnect::new(client_config),
      )
      .unwrap()
  } else {
    info!("using unencrypted database connection");
    pool
      .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
      .unwrap()
  };

  Ok(pool)
}
