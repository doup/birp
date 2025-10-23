use bevy_remote::builtin_methods::{BrpGetComponentsResponse, BrpQueryRow};
use entity_kind::KIND_COMPONENTS;
use futures::future::join_all;
use serde::Deserialize;
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
pub use serde_json::{Value, from_value, json};

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
            .call("world.list_components", Some(json!({ "entity": id })))
            .await?;

        let entity = self
            .call(
                "world.get_components",
                Some(json!({
                    "entity": id,
                    "components": components
                })),
            )
            .await?;

        Ok((id, from_value::<BrpGetComponentsResponse>(entity)?).into())
    }

    pub async fn get_many(&self, ids: Vec<Entity>) -> Result<Vec<EntityItem>, ClientError> {
        let components = {
            let mut components = vec![component::NAME, component::CHILDREN, component::CHILD_OF];
            components.extend_from_slice(&KIND_COMPONENTS);
            components
        };

        let futures = ids
            .into_iter()
            .map(async |id| -> Result<EntityItem, ClientError> {
                let entity = self
                    .call(
                        "world.get_components",
                        Some(json!({
                            "entity": id,
                            "components": components
                        })),
                    )
                    .await?;

                Ok((id, from_value::<BrpGetComponentsResponse>(entity)?).into())
            });

        // Wait for all futures to complete
        let results = join_all(futures).await;

        results.into_iter().collect()
    }

    pub async fn get_children(
        &self,
        parent_id: Option<Entity>,
    ) -> Result<Vec<EntityItem>, ClientError> {
        let entities = match parent_id {
            Some(id) => {
                // Get the parent entity so we can get its children IDs
                let parent = self
                    .call(
                        "world.get_components",
                        Some(json!({
                            "entity": id,
                            "components": [component::CHILDREN]
                        })),
                    )
                    .await?;

                let parent: EntityItem =
                    (id, from_value::<BrpGetComponentsResponse>(parent)?).into();

                parent.children()
            }
            None => {
                let res = self
                    .call(
                        "world.query",
                        Some(json!({
                            "data": {},
                            "filter": {
                                "without": [component::CHILD_OF]
                            },
                        })),
                    )
                    .await?;

                let res = from_value::<Vec<BrpQueryRow>>(res)?;

                res.into_iter().map(|row| row.entity).collect()
            }
        };

        self.get_many(entities).await
    }

    pub async fn get_resource(&self, resource: String) -> Result<Value, ClientError> {
        let res = self
            .call(
                "world.get_resources",
                Some(json!({
                    "resource": resource,
                })),
            )
            .await?;

        Ok(res)
    }

    pub async fn get_schema(&self) -> Result<BTreeMap<String, JsonSchemaBevyType>, ClientError> {
        let res = self.call("registry.schema", None).await?;
        let schema = from_value::<BTreeMap<String, JsonSchemaBevyType>>(res)?;
        Ok(schema)
    }

    pub async fn list_resources(&self) -> Result<Vec<String>, ClientError> {
        let res = self.call("world.list_resources", None).await?;
        let resources = from_value::<Vec<String>>(res)?;
        Ok(resources)
    }

    pub async fn mutate_component(
        &self,
        id: Entity,
        component: String,
        path: String,
        value: Value,
    ) -> Result<(), ClientError> {
        self.call(
            "world.mutate_components",
            Some(json!({
                "entity": id,
                "component": component,
                "path": path,
                "value": value,
            })),
        )
        .await?;

        Ok(())
    }

    pub async fn ping(&self) -> Result<(), ClientError> {
        self.call("rpc.discover", None).await?;
        Ok(())
    }
}
