use bevy_remote::builtin_methods::{BrpGetResponse, BrpQueryRow};
use entity_kind::KIND_COMPONENTS;
use futures::future::join_all;
use serde::Deserialize;
use serde_json::{from_value, json};
use std::{
    collections::BTreeMap,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};
use thiserror::Error;

pub mod component;
mod entity_item;
mod entity_kind;

// (Re)Exports
pub use bevy_ecs::entity::Entity;
pub use bevy_remote::schemas::json_schema::{JsonSchemaBevyType, SchemaKind, SchemaType};
pub use entity_item::EntityItem;
pub use entity_kind::EntityKind;
pub use serde_json::Value;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("server error: {0}")]
    ServerError(BrpError),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BrpResponse<T> {
    Success { id: u32, result: T },
    Error { id: u32, error: BrpError },
}

// TODO: Copied from `bevy_remote`, use Bevy type once it implements `Display`
/// An error a request might return.
#[derive(Debug, Deserialize, Clone)]
pub struct BrpError {
    /// Defines the general type of the error.
    pub code: i16,
    /// Short, human-readable description of the error.
    pub message: String,
    /// Optional additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// TODO(bevy_remote): Move to Bevy code?
impl fmt::Display for BrpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (code={})", self.message, self.code)
    }
}

#[derive(Clone)]
pub struct BrpClient {
    call_id: Arc<AtomicU32>,
    client: reqwest::Client,
    url: Arc<String>,
}

impl PartialEq for BrpClient {
    fn eq(&self, other: &Self) -> bool {
        // Compare the `url` field (wrapped in `Arc`) for equality
        Arc::ptr_eq(&self.url, &other.url) || *self.url == *other.url
    }
}

impl BrpClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            call_id: Arc::new(AtomicU32::new(0)),
            client: reqwest::Client::new(),
            url: Arc::new(url.into()),
        }
    }

    fn next_call_id(&self) -> u32 {
        self.call_id.fetch_add(1, Ordering::SeqCst)
    }

    async fn call(&self, method: &str, params: Option<Value>) -> Result<Value, ClientError> {
        let call_id = self.next_call_id();
        let res = self
            .client
            .post(self.url.as_str())
            .json(&json!({
                "method": method,
                "id": call_id,
                "jsonrpc": "2.0",
                "params": params
            }))
            .send()
            .await?;

        let res = res.json::<BrpResponse<Value>>().await?;

        match res {
            BrpResponse::Success { result, .. } => Ok(result),
            BrpResponse::Error { error, .. } => Err(ClientError::ServerError(error)),
        }
    }

    pub async fn get(&self, id: Entity) -> Result<EntityItem, ClientError> {
        let components = self
            .call("bevy/list", Some(json!({ "entity": id })))
            .await?;

        let entity = self
            .call(
                "bevy/get",
                Some(json!({
                    "entity": id,
                    "components": components
                })),
            )
            .await?;

        Ok((id, from_value::<BrpGetResponse>(entity)?).into())
    }

    pub async fn get_children(
        &self,
        parent_id: Option<Entity>,
    ) -> Result<Vec<EntityItem>, ClientError> {
        match parent_id {
            Some(id) => {
                // Get the parent entity so we can get its children IDs
                let parent = self
                    .call(
                        "bevy/get",
                        Some(json!({
                            "entity": id,
                            "components": [component::CHILDREN]
                        })),
                    )
                    .await?;

                let parent: EntityItem = (id, from_value::<BrpGetResponse>(parent)?).into();

                // Fetch all the children
                let components = {
                    let mut components =
                        vec![component::NAME, component::CHILDREN, component::CHILD_OF];
                    components.extend_from_slice(&KIND_COMPONENTS);
                    components
                };
                let futures = parent.children().into_iter().map(
                    async |id| -> Result<EntityItem, ClientError> {
                        let entity = self
                            .call(
                                "bevy/get",
                                Some(json!({
                                    "entity": id,
                                    "components": components
                                })),
                            )
                            .await?;

                        Ok((id, from_value::<BrpGetResponse>(entity)?).into())
                    },
                );

                // Wait for all futures to complete
                let results = join_all(futures).await;

                // Collect the results into a single `Result`
                let entities: Result<Vec<EntityItem>, ClientError> = results.into_iter().collect();

                entities
            }
            None => {
                let res = self
                    .call(
                        "bevy/query",
                        Some(json!({
                            "data": {
                                "option": [
                                    component::NAME,
                                    component::CHILDREN,
                                    component::CHILD_OF,
                                ],
                                "has": KIND_COMPONENTS,
                            },
                            "filter": {
                                "without": [component::CHILD_OF]
                            },
                        })),
                    )
                    .await?;

                let res = from_value::<Vec<BrpQueryRow>>(res)?;

                Ok(res.into_iter().map(Into::into).collect())
            }
        }
    }

    pub async fn get_schema(&self) -> Result<BTreeMap<String, JsonSchemaBevyType>, ClientError> {
        let res = self.call("bevy/registry/schema", None).await?;
        let schema = from_value::<BTreeMap<String, JsonSchemaBevyType>>(res)?;

        let resources = self.list_resources().await?;
        println!("Resources: {:?}", resources);

        Ok(schema)
    }

    pub async fn ping(&self) -> Result<(), ClientError> {
        self.call("rpc.discover", None).await?;
        Ok(())
    }
}
