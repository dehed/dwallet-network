// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: BSD-3-Clause-Clear

//! The Ika Rust SDK
//!
//! It aims at providing a similar SDK functionality like the one existing for
//! [TypeScript](https://github.com/MystenLabs/sui/tree/main/sdk/typescript/).
//! Ika Rust SDK builds on top of the [JSON RPC API](https://docs.ika.io/ika-jsonrpc)
//! and therefore many of the return types are the ones specified in [ika_types].
//!
//! The API is split in several parts corresponding to different functionalities
//! as following:
//! * [CoinReadApi] - provides read-only functions to work with the coins
//! * [EventApi] - provides event related functions functions to
//! * [GovernanceApi] - provides functionality related to staking
//! * [QuorumDriverApi] - provides functionality to execute a transaction
//!     block and submit it to the fullnode(s)
//! * [ReadApi] - provides functions for retrieving data about different
//!     objects and transactions
//! * <a href="../ika_transaction_builder/struct.TransactionBuilder.html" title="struct ika_transaction_builder::TransactionBuilder">TransactionBuilder</a> - provides functions for building transactions
//!
//! # Usage
//! The main way to interact with the API is through the [IkaClientBuilder],
//! which returns a [IkaClient] object from which the user can access the
//! various APIs.
//!
//! ## Getting Started
//! Add the Rust SDK to the project by running `cargo add ika-sdk` in the root
//! folder of your Rust project.
//!
//! The main building block for the Ika Rust SDK is the [IkaClientBuilder],
//! which provides a simple and straightforward way of connecting to a Ika
//! network and having access to the different available APIs.
//!
//! A simple example that connects to a running Ika local network,
//! the Ika devnet, and the Ika testnet is shown below.
//! To successfully run this program, make sure to spin up a local
//! network with a local validator, a fullnode, and a faucet server
//! (see [here](https://github.com/stefan-ika-io/ika/tree/rust_sdk_api_examples/crates/ika-sdk/examples#preqrequisites) for more information).
//!
//! ```rust,no_run
//! use ika_sdk::IkaClientBuilder;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!
//!     let ika = IkaClientBuilder::default()
//!         .build("http://127.0.0.1:9000") // provide the Ika network URL
//!         .await?;
//!     println!("Ika local network version: {:?}", ika.api_version());
//!
//!     // local Ika network, same result as above except using the dedicated function
//!     let ika_local = IkaClientBuilder::default().build_localnet().await?;
//!     println!("Ika local network version: {:?}", ika_local.api_version());
//!
//!     // Ika devnet running at `https://fullnode.devnet.io:443`
//!     let ika_devnet = IkaClientBuilder::default().build_devnet().await?;
//!     println!("Ika devnet version: {:?}", ika_devnet.api_version());
//!
//!     // Ika testnet running at `https://testnet.devnet.io:443`
//!     let ika_testnet = IkaClientBuilder::default().build_testnet().await?;
//!     println!("Ika testnet version: {:?}", ika_testnet.api_version());
//!     Ok(())
//!
//! }
//! ```
//!
//! ## Examples
//!
//! For detailed examples, please check the APIs docs and the examples folder
//! in the [main repository](https://github.com/MystenLabs/sui/tree/main/crates/ika-sdk/examples).

use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use base64::Engine;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HeaderMap, HeaderValue, HttpClient, HttpClientBuilder};
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use serde_json::Value;

use move_core_types::language_storage::StructTag;
pub use ika_types as types;

use crate::apis::{CoinReadApi, EventApi, GovernanceApi, QuorumDriverApi, ReadApi};
use crate::error::{Error, IkaRpcResult};

pub mod apis;
pub mod error;
pub mod json_rpc_error;
pub mod ika_client_config;
pub mod wallet_context;

pub const IKA_COIN_TYPE: &str = "0x2::ika::IKA";
pub const IKA_LOCAL_NETWORK_URL: &str = "http://127.0.0.1:9000";
pub const IKA_LOCAL_NETWORK_URL_0: &str = "http://0.0.0.0:9000";
pub const IKA_LOCAL_NETWORK_GAS_URL: &str = "http://127.0.0.1:5003/gas";
pub const IKA_DEVNET_URL: &str = "https://fullnode.devnet.ika.io:443";
pub const IKA_TESTNET_URL: &str = "https://fullnode.testnet.ika.io:443";

/// A Ika client builder for connecting to the Ika network
///
/// By default the `maximum concurrent requests` is set to 256 and
/// the `request timeout` is set to 60 seconds. These can be adjusted using the
/// `max_concurrent_requests` function, and the `request_timeout` function.
/// If you use the WebSocket, consider setting the `ws_ping_interval` field to a
/// value of your choice to prevent the inactive WS subscription being
/// disconnected due to proxy timeout.
///
/// # Examples
///
/// ```rust,no_run
/// use ika_sdk::IkaClientBuilder;
/// #[tokio::main]
/// async fn main() -> Result<(), anyhow::Error> {
///     let ika = IkaClientBuilder::default()
///         .build("http://127.0.0.1:9000")
///         .await?;
///
///     println!("Ika local network version: {:?}", ika.api_version());
///     Ok(())
/// }
/// ```
pub struct IkaClientBuilder {
    request_timeout: Duration,
    max_concurrent_requests: usize,
    ws_url: Option<String>,
    ws_ping_interval: Option<Duration>,
    basic_auth: Option<(String, String)>,
}

impl Default for IkaClientBuilder {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(60),
            max_concurrent_requests: 256,
            ws_url: None,
            ws_ping_interval: None,
            basic_auth: None,
        }
    }
}

impl IkaClientBuilder {
    /// Set the request timeout to the specified duration
    pub fn request_timeout(mut self, request_timeout: Duration) -> Self {
        self.request_timeout = request_timeout;
        self
    }

    /// Set the max concurrent requests allowed
    pub fn max_concurrent_requests(mut self, max_concurrent_requests: usize) -> Self {
        self.max_concurrent_requests = max_concurrent_requests;
        self
    }

    /// Set the WebSocket URL for the Ika network
    pub fn ws_url(mut self, url: impl AsRef<str>) -> Self {
        self.ws_url = Some(url.as_ref().to_string());
        self
    }

    /// Set the WebSocket ping interval
    pub fn ws_ping_interval(mut self, duration: Duration) -> Self {
        self.ws_ping_interval = Some(duration);
        self
    }

    /// Set the basic auth credentials for the HTTP client
    pub fn basic_auth(mut self, username: impl AsRef<str>, password: impl AsRef<str>) -> Self {
        self.basic_auth = Some((username.as_ref().to_string(), password.as_ref().to_string()));
        self
    }

    /// Returns a [IkaClient] object connected to the Ika network running at the URI provided.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ika_sdk::IkaClientBuilder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), anyhow::Error> {
    ///     let ika = IkaClientBuilder::default()
    ///         .build("http://127.0.0.1:9000")
    ///         .await?;
    ///
    ///     println!("Ika local version: {:?}", ika.api_version());
    ///     Ok(())
    /// }
    /// ```
    pub async fn build(self, http: impl AsRef<str>) -> IkaRpcResult<IkaClient> {
        let client_version = env!("CARGO_PKG_VERSION");
        let mut headers = HeaderMap::new();
        headers.insert(
            CLIENT_TARGET_API_VERSION_HEADER,
            // in rust, the client version is the same as the target api version
            HeaderValue::from_static(client_version),
        );
        headers.insert(
            CLIENT_SDK_VERSION_HEADER,
            HeaderValue::from_static(client_version),
        );
        headers.insert(CLIENT_SDK_TYPE_HEADER, HeaderValue::from_static("rust"));

        if let Some((username, password)) = self.basic_auth {
            let auth = base64::engine::general_purpose::STANDARD
                .encode(format!("{}:{}", username, password));
            headers.insert(
                "authorization",
                // reqwest::header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {}", auth)).unwrap(),
            );
        }

        let ws = if let Some(url) = self.ws_url {
            let mut builder = WsClientBuilder::default()
                .max_request_body_size(2 << 30)
                .max_concurrent_requests(self.max_concurrent_requests)
                .set_headers(headers.clone())
                .request_timeout(self.request_timeout);

            if let Some(duration) = self.ws_ping_interval {
                builder = builder.ping_interval(duration)
            }

            builder.build(url).await.ok()
        } else {
            None
        };

        let http = HttpClientBuilder::default()
            .max_request_body_size(2 << 30)
            .max_concurrent_requests(self.max_concurrent_requests)
            .set_headers(headers.clone())
            .request_timeout(self.request_timeout)
            .build(http)?;

        let info = Self::get_server_info(&http, &ws).await?;

        let rpc = RpcClient { http, ws, info };
        let api = Arc::new(rpc);
        let read_api = Arc::new(ReadApi::new(api.clone()));
        let quorum_driver_api = QuorumDriverApi::new(api.clone());
        let event_api = EventApi::new(api.clone());
        let transaction_builder = TransactionBuilder::new(read_api.clone());
        let coin_read_api = CoinReadApi::new(api.clone());
        let governance_api = GovernanceApi::new(api.clone());

        Ok(IkaClient {
            api,
            transaction_builder,
            read_api,
            coin_read_api,
            event_api,
            quorum_driver_api,
            governance_api,
        })
    }

    /// Returns a [IkaClient] object that is ready to interact with the local
    /// development network (by default it expects the Ika network to be
    /// up and running at `127.0.0.1:9000`).
    ///
    /// For connecting to a custom URI, use the `build` function instead.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ika_sdk::IkaClientBuilder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), anyhow::Error> {
    ///     let ika = IkaClientBuilder::default()
    ///         .build_localnet()
    ///         .await?;
    ///
    ///     println!("Ika local version: {:?}", ika.api_version());
    ///     Ok(())
    /// }
    /// ```
    pub async fn build_localnet(self) -> IkaRpcResult<IkaClient> {
        self.build(IKA_LOCAL_NETWORK_URL).await
    }

    /// Returns a [IkaClient] object that is ready to interact with the Ika devnet.
    ///
    /// For connecting to a custom URI, use the `build` function instead..
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ika_sdk::IkaClientBuilder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), anyhow::Error> {
    ///     let ika = IkaClientBuilder::default()
    ///         .build_devnet()
    ///         .await?;
    ///
    ///     println!("{:?}", ika.api_version());
    ///     Ok(())
    /// }
    /// ```
    pub async fn build_devnet(self) -> IkaRpcResult<IkaClient> {
        self.build(IKA_DEVNET_URL).await
    }

    /// Returns a [IkaClient] object that is ready to interact with the Ika testnet.
    ///
    /// For connecting to a custom URI, use the `build` function instead.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ika_sdk::IkaClientBuilder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), anyhow::Error> {
    ///     let ika = IkaClientBuilder::default()
    ///         .build_testnet()
    ///         .await?;
    ///
    ///     println!("{:?}", ika.api_version());
    ///     Ok(())
    /// }
    /// ```
    pub async fn build_testnet(self) -> IkaRpcResult<IkaClient> {
        self.build(IKA_TESTNET_URL).await
    }

    /// Return the server information as a `ServerInfo` structure.
    ///
    /// Fails with an error if it cannot call the RPC discover.
    async fn get_server_info(
        http: &HttpClient,
        ws: &Option<WsClient>,
    ) -> Result<ServerInfo, Error> {
        let rpc_spec: Value = http.request("rpc.discover", rpc_params![]).await?;
        let version = rpc_spec
            .pointer("/info/version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::DataError("Fail parsing server version from rpc.discover endpoint.".into())
            })?;
        let rpc_methods = Self::parse_methods(&rpc_spec)?;

        let subscriptions = if let Some(ws) = ws {
            match ws.request("rpc.discover", rpc_params![]).await {
                Ok(rpc_spec) => Self::parse_methods(&rpc_spec)?,
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };
        Ok(ServerInfo {
            rpc_methods,
            subscriptions,
            version: version.to_string(),
        })
    }

    fn parse_methods(server_spec: &Value) -> Result<Vec<String>, Error> {
        let methods = server_spec
            .pointer("/methods")
            .and_then(|methods| methods.as_array())
            .ok_or_else(|| {
                Error::DataError(
                    "Fail parsing server information from rpc.discover endpoint.".into(),
                )
            })?;

        Ok(methods
            .iter()
            .flat_map(|method| method["name"].as_str())
            .map(|s| s.into())
            .collect())
    }
}

/// IkaClient is the basic type that provides all the necessary abstractions for interacting with the Ika network.
///
/// # Usage
///
/// Use [IkaClientBuilder] to build a [IkaClient].
///
/// # Examples
///
/// ```rust,no_run
/// use ika_sdk::types::base_types::IkaAddress;
/// use ika_sdk::IkaClientBuilder;
/// use std::str::FromStr;
///
/// #[tokio::main]
/// async fn main() -> Result<(), anyhow::Error> {
///     let ika = IkaClientBuilder::default()
///      .build("http://127.0.0.1:9000")
///      .await?;
///
///     println!("{:?}", ika.available_rpc_methods());
///     println!("{:?}", ika.available_subscriptions());
///     println!("{:?}", ika.api_version());
///
///     let address = IkaAddress::from_str("0x0000....0000")?;
///     let owned_objects = ika
///        .read_api()
///        .get_owned_objects(address, None, None, None)
///        .await?;
///
///     println!("{:?}", owned_objects);
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct IkaClient {
    api: Arc<RpcClient>,
    transaction_builder: TransactionBuilder,
    read_api: Arc<ReadApi>,
    coin_read_api: CoinReadApi,
    event_api: EventApi,
    quorum_driver_api: QuorumDriverApi,
    governance_api: GovernanceApi,
}

pub(crate) struct RpcClient {
    http: HttpClient,
    ws: Option<WsClient>,
    info: ServerInfo,
}

impl Debug for RpcClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RPC client. Http: {:?}, Websocket: {:?}",
            self.http, self.ws
        )
    }
}

/// ServerInfo contains all the useful information regarding the API version, the available RPC calls, and subscriptions.
struct ServerInfo {
    rpc_methods: Vec<String>,
    subscriptions: Vec<String>,
    version: String,
}

impl IkaClient {
    /// Returns a list of RPC methods supported by the node the client is connected to.
    pub fn available_rpc_methods(&self) -> &Vec<String> {
        &self.api.info.rpc_methods
    }

    /// Returns a list of streaming/subscription APIs supported by the node the client is connected to.
    pub fn available_subscriptions(&self) -> &Vec<String> {
        &self.api.info.subscriptions
    }

    /// Returns the API version information as a string.
    ///
    /// The format of this string is `<major>.<minor>.<patch>`, e.g., `1.6.0`,
    /// and it is retrieved from the OpenRPC specification via the discover service method.
    pub fn api_version(&self) -> &str {
        &self.api.info.version
    }

    /// Verifies if the API version matches the server version and returns an error if they do not match.
    pub fn check_api_version(&self) -> IkaRpcResult<()> {
        let server_version = self.api_version();
        let client_version = env!("CARGO_PKG_VERSION");
        if server_version != client_version {
            return Err(Error::ServerVersionMismatch {
                client_version: client_version.to_string(),
                server_version: server_version.to_string(),
            });
        };
        Ok(())
    }

    /// Returns a reference to the coin read API.
    pub fn coin_read_api(&self) -> &CoinReadApi {
        &self.coin_read_api
    }

    /// Returns a reference to the event API.
    pub fn event_api(&self) -> &EventApi {
        &self.event_api
    }

    /// Returns a reference to the governance API.
    pub fn governance_api(&self) -> &GovernanceApi {
        &self.governance_api
    }

    /// Returns a reference to the quorum driver API.
    pub fn quorum_driver_api(&self) -> &QuorumDriverApi {
        &self.quorum_driver_api
    }

    /// Returns a reference to the read API.
    pub fn read_api(&self) -> &ReadApi {
        &self.read_api
    }

    /// Returns a reference to the transaction builder API.
    pub fn transaction_builder(&self) -> &TransactionBuilder {
        &self.transaction_builder
    }

    /// Returns a reference to the underlying http client.
    pub fn http(&self) -> &HttpClient {
        &self.api.http
    }

    /// Returns a reference to the underlying WebSocket client, if any.
    pub fn ws(&self) -> Option<&WsClient> {
        self.api.ws.as_ref()
    }
}

#[async_trait]
impl DataReader for ReadApi {
    async fn get_owned_objects(
        &self,
        address: IkaAddress,
        object_type: StructTag,
    ) -> Result<Vec<ObjectInfo>, anyhow::Error> {
        let mut result = vec![];
        let query = Some(IkaObjectResponseQuery {
            filter: Some(IkaObjectDataFilter::StructType(object_type)),
            options: Some(
                IkaObjectDataOptions::new()
                    .with_previous_transaction()
                    .with_type()
                    .with_owner(),
            ),
        });

        let mut has_next = true;
        let mut cursor = None;

        while has_next {
            let ObjectsPage {
                data,
                next_cursor,
                has_next_page,
            } = self
                .get_owned_objects(address, query.clone(), cursor, None)
                .await?;
            result.extend(
                data.iter()
                    .map(|r| r.clone().try_into())
                    .collect::<Result<Vec<_>, _>>()?,
            );
            cursor = next_cursor;
            has_next = has_next_page;
        }
        Ok(result)
    }

    async fn get_object_with_options(
        &self,
        object_id: ObjectID,
        options: IkaObjectDataOptions,
    ) -> Result<IkaObjectResponse, anyhow::Error> {
        Ok(self.get_object_with_options(object_id, options).await?)
    }

    /// Returns the computation price per unit size as a u64 or an error otherwise
    async fn get_computation_price_per_unit_size(&self) -> Result<u64, anyhow::Error> {
        Ok(self.get_computation_price_per_unit_size().await?)
    }
}
