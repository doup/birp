// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let body = BrpRequest {
//         jsonrpc: String::from("2.0"),
//         method: String::from("bevy/list"),
//         // id: Some(serde_json::to_value(1)?),
//         id: Some(json!(1)),
//         params: None,
//     };

//     let client = reqwest::Client::new();
//     let res = client
//         .post("http://127.0.0.1:15702")
//         .json(&body)
//         // .json(&json!({
//         //     "method": "bevy/list",
//         //     "id": 1,
//         //     "jsonrpc": "2.0",
//         //     // "params": json!({})
//         // }))
//         .send()
//         .await?;

//     println!("{res:#?}");
//     let body = res.json::<serde_json::Value>().await?;
//     println!("{body:#?}");

//     let body = client
//         .post("http://127.0.0.1:15702")
//         .json(&json!({
//             "method": "bevy/query",
//             "id": 1,
//             "jsonrpc": "2.0",
//             "params": {
//                 "data": {
//                     "option": [
//                         "bevy_ecs::name::Name",
//                     ],
//                     "has": [
//                         "bevy_render::camera::camera::Camera",
//                         // "bevy_ecs::observer::ObserverState",
//                     ]
//                 },
//                 "filter": { "without": ["bevy_ecs::hierarchy::ChildOf"] },
//             }
//         }))
//         .send()
//         .await?
//         .json::<serde_json::Value>()
//         .await?;

//     println!("{body:#?}");

//     Ok(())
// }

use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct BrpResponseList<T> {
    pub id: u32,
    pub result: Vec<T>,
}

#[derive(Debug)]
pub enum EntityKind {
    Camera,
    Light,
    Observer,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct BrpTreeEntity {
    pub entity: u64,
    pub components: Option<HashMap<String, String>>,
    pub has: HashMap<String, bool>,
}

impl BrpTreeEntity {
    fn has_component(&self, component: &str) -> bool {
        self.has.get(component).unwrap_or(&false).clone()
    }

    pub fn kind(&self) -> EntityKind {
        if self.has_component("bevy_render::camera::camera::Camera") {
            EntityKind::Camera
        } else if self.has_component("bevy_pbr::light::directional_light::DirectionalLight")
            || self.has_component("bevy_pbr::light::point_light::PointLight")
            || self.has_component("bevy_pbr::light::spot_light::SpotLight")
        {
            EntityKind::Light
        } else if self.has_component("bevy_ecs::observer::ObserverState") {
            EntityKind::Observer
        } else {
            EntityKind::Unknown
        }
    }

    pub fn name(&self) -> String {
        self.components
            .as_ref()
            .and_then(|c| c.get("bevy_ecs::name::Name"))
            .cloned()
            .unwrap_or_default()
    }
}

pub async fn get_root_entities() -> Result<BrpResponseList<BrpTreeEntity>, reqwest::Error> {
    let client = reqwest::Client::new();

    client
        .post("http://127.0.0.1:15702")
        .json(&json!({
            "method": "bevy/query",
            "id": 1,
            "jsonrpc": "2.0",
            "params": {
                "data": {
                    "option": [
                        "bevy_ecs::name::Name",
                    ],
                    "has": [
                        "bevy_render::camera::camera::Camera",
                        "bevy_pbr::light::directional_light::DirectionalLight",
                        "bevy_pbr::light::point_light::PointLight",
                        "bevy_pbr::light::spot_light::SpotLight",
                        // "bevy_ecs::observer::ObserverState",
                    ]
                },
                "filter": { "without": ["bevy_ecs::hierarchy::ChildOf"] },
            }
        }))
        .send()
        .await
        // rewrite as combinator on result?
        .unwrap()
        .json::<BrpResponseList<BrpTreeEntity>>()
        .await
}
