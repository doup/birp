use futures::future::join_all;
use serde::{Deserialize, Deserializer, de::Error};
use serde_json::{Value, from_value, json};
use std::{
    collections::HashMap,
    fmt::{self},
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};
use thiserror::Error;

const COMPONENT_CAMERA: &str = "bevy_render::camera::camera::Camera";
const COMPONENT_CHILD_OF: &str = "bevy_ecs::hierarchy::ChildOf";
const COMPONENT_CHILDREN: &str = "bevy_ecs::hierarchy::Children";
const COMPONENT_LIGHT_DIRECTIONAL: &str = "bevy_pbr::light::directional_light::DirectionalLight";
const COMPONENT_LIGHT_POINT: &str = "bevy_pbr::light::point_light::PointLight";
const COMPONENT_LIGHT_SPOT: &str = "bevy_pbr::light::spot_light::SpotLight";
const COMPONENT_MESH_2D: &str = "bevy_render::mesh::components::Mesh2d";
const COMPONENT_MESH_3D: &str = "bevy_render::mesh::components::Mesh3d";
const COMPONENT_NAME: &str = "bevy_ecs::name::Name";
const COMPONENT_WINDOW: &str = "bevy_window::window::Window";

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

#[derive(Debug, Deserialize)]
pub struct BrpError {
    pub code: i16,
    pub message: String,
    // Optional additional error data
    pub data: Option<Value>,
}

impl fmt::Display for BrpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (code={})", self.message, self.code)
    }
}

#[derive(Debug)]
pub enum EntityKind {
    Camera,
    Light,
    Mesh2d,
    Mesh3d,
    Observer,
    Unknown,
    Window,
}

#[derive(Debug, Deserialize)]
pub struct HierarchyEntity {
    #[serde(rename = "entity")]
    pub id: u64,
    pub components: HashMap<String, Value>,
    pub has: Option<HashMap<String, bool>>,
}

impl HierarchyEntity {
    pub fn children(&self) -> Vec<u64> {
        self.get_component_as::<Vec<u64>>(COMPONENT_CHILDREN)
            .unwrap_or_default()
    }

    pub fn get_component_as<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.components
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    fn has_component(&self, component: &str) -> bool {
        match self.has {
            Some(ref has) => has
                .get(component)
                .unwrap_or(&self.components.contains_key(component))
                .clone(),
            None => self.components.contains_key(component),
        }
    }

    pub fn kind(&self) -> EntityKind {
        if self.has_component(COMPONENT_CAMERA) {
            EntityKind::Camera
        } else if self.has_component(COMPONENT_LIGHT_DIRECTIONAL)
            || self.has_component(COMPONENT_LIGHT_POINT)
            || self.has_component(COMPONENT_LIGHT_SPOT)
        {
            EntityKind::Light
        } else if self.has_component(COMPONENT_MESH_2D) {
            EntityKind::Mesh2d
        } else if self.has_component(COMPONENT_MESH_3D) {
            EntityKind::Mesh3d
        } else if self.has_component(COMPONENT_WINDOW) {
            EntityKind::Window
        } else if self.has_component("bevy_ecs::observer::ObserverState") {
            EntityKind::Observer
        } else {
            EntityKind::Unknown
        }
    }

    pub fn name(&self) -> String {
        self.get_component_as::<String>(COMPONENT_NAME)
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct HierarchyParentEntity {
    pub children: Vec<u64>,
}

impl<'de> Deserialize<'de> for HierarchyParentEntity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TempEntity {
            components: HashMap<String, Value>,
        }

        let mut temp = TempEntity::deserialize(deserializer)?;
        let mut children = Vec::new();

        // Extract list of children from components
        if let Some(value) = temp.components.remove(COMPONENT_CHILDREN) {
            if let Ok(extracted_children) = serde_json::from_value::<Vec<u64>>(value) {
                children = extracted_children;
            }
        }

        Ok(HierarchyParentEntity { children })
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

    async fn call(&self, method: &str, params: Value) -> Result<Value, ClientError> {
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

    pub async fn get_children(
        &self,
        parent_id: Option<u64>,
    ) -> Result<Vec<HierarchyEntity>, ClientError> {
        match parent_id {
            Some(id) => {
                // Get the parent entity so we can get its children IDs
                let parent = self
                    .call(
                        "bevy/get",
                        json!({
                            "entity": id,
                            "components": [COMPONENT_CHILDREN]
                        }),
                    )
                    .await?;

                let parent = from_value::<HierarchyParentEntity>(parent)?;

                // Fetch all the children
                let futures = parent.children.into_iter().map(
                    async |id| -> Result<HierarchyEntity, ClientError> {
                        let entity = self
                            .call(
                                "bevy/get",
                                json!({
                                    "entity": id,
                                    "components": [
                                        COMPONENT_NAME,
                                        COMPONENT_CHILDREN,
                                        COMPONENT_CHILD_OF,
                                        // Components to find-out the "kind"
                                        COMPONENT_CAMERA,
                                        COMPONENT_LIGHT_DIRECTIONAL,
                                        COMPONENT_LIGHT_POINT,
                                        COMPONENT_LIGHT_SPOT,
                                        COMPONENT_MESH_2D,
                                        COMPONENT_MESH_3D,
                                        COMPONENT_WINDOW,
                                    ]
                                }),
                            )
                            .await?;

                        let components = entity
                            .get("components")
                            .and_then(|v| v.as_object())
                            .map(|map| map.clone().into_iter().collect())
                            .ok_or(ClientError::ParseError(serde_json::Error::custom(
                                "failed to parse 'components' property",
                            )))?;

                        let has_map = entity.get("has").and_then(|v| v.as_object());
                        let has = match has_map {
                            Some(map) => Some(
                                map.iter()
                                    .map(|(k, v)| (k.clone(), v.as_bool().unwrap_or(false)))
                                    .collect::<HashMap<String, bool>>(),
                            ),
                            None => None,
                        };

                        Ok(HierarchyEntity {
                            id,
                            components,
                            has,
                        })
                    },
                );

                // Wait for all futures to complete
                let results = join_all(futures).await;

                // Collect the results into a single `Result`
                let entities: Result<Vec<HierarchyEntity>, ClientError> =
                    results.into_iter().collect();

                entities
            }
            None => {
                let res = self
                    .call(
                        "bevy/query",
                        json!({
                            "data": {
                                "option": [
                                    COMPONENT_NAME,
                                    COMPONENT_CHILDREN,
                                    COMPONENT_CHILD_OF,
                                ],
                                "has": [
                                    COMPONENT_CAMERA,
                                    COMPONENT_LIGHT_DIRECTIONAL,
                                    COMPONENT_LIGHT_POINT,
                                    COMPONENT_LIGHT_SPOT,
                                    COMPONENT_MESH_2D,
                                    COMPONENT_MESH_3D,
                                    COMPONENT_WINDOW,
                                ]
                            },
                            "filter": {
                                "without": [COMPONENT_CHILD_OF]
                            },
                        }),
                    )
                    .await?;

                let res = from_value::<Vec<HierarchyEntity>>(res)?;

                Ok(res)
            }
        }
    }

    pub async fn ping(&self) -> Result<(), ClientError> {
        self.call("rpc.discover", json!({})).await?;
        Ok(())
    }
}
