use island_model::{ActivityKind, IslandMode, IslandSnapshot};
use js_sys::{Function, Reflect};
use serde::Serialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;

pub const SNAPSHOT_EVENT: &str = "open-island://snapshot";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn tauri_invoke(command: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"], js_name = listen)]
    async fn tauri_listen(event: &str, callback: &Function) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

#[derive(Debug, Clone)]
pub struct BridgeError(String);

impl std::fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub struct EventSubscription {
    unlisten: Function,
    _callback: Closure<dyn FnMut(JsValue)>,
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        let _ = self.unlisten.call0(&JsValue::NULL);
    }
}

pub async fn frontend_ready() -> Result<IslandSnapshot, BridgeError> {
    invoke("frontend_ready", &EmptyArgs {}).await
}

pub async fn first_frame_ready() -> Result<(), BridgeError> {
    invoke("first_frame_ready", &EmptyArgs {}).await
}

pub async fn toggle_expansion() -> Result<(), BridgeError> {
    invoke("toggle_expansion", &EmptyArgs {}).await
}

pub async fn dismiss_activity(activity_id: &str) -> Result<(), BridgeError> {
    invoke("dismiss_activity", &ActivityArgs { activity_id }).await
}

pub async fn invoke_activity_action(activity_id: &str, action_id: &str) -> Result<(), BridgeError> {
    invoke(
        "invoke_activity_action",
        &ActionArgs {
            activity_id,
            action_id,
        },
    )
    .await
}

pub async fn animation_completed(
    transition_id: u64,
    final_mode: IslandMode,
) -> Result<(), BridgeError> {
    invoke(
        "animation_completed",
        &AnimationArgs {
            transition_id,
            final_mode,
        },
    )
    .await
}

#[allow(dead_code)]
pub async fn run_demo(kind: ActivityKind) -> Result<(), BridgeError> {
    invoke("run_demo", &DemoArgs { kind, failed: None }).await
}

pub async fn subscribe_to_snapshots(
    callback: impl Fn(IslandSnapshot) + 'static,
) -> Result<EventSubscription, BridgeError> {
    let closure = Closure::wrap(Box::new(move |event: JsValue| {
        let payload = Reflect::get(&event, &JsValue::from_str("payload"));
        match payload
            .map_err(js_error)
            .and_then(|value| serde_wasm_bindgen::from_value(value).map_err(serde_error))
        {
            Ok(snapshot) => callback(snapshot),
            Err(error) => log_error(&error),
        }
    }) as Box<dyn FnMut(JsValue)>);
    let unlisten = tauri_listen(SNAPSHOT_EVENT, closure.as_ref().unchecked_ref())
        .await
        .map_err(js_error)?
        .dyn_into::<Function>()
        .map_err(|_| BridgeError("event listener did not return an unsubscribe function".into()))?;
    Ok(EventSubscription {
        unlisten,
        _callback: closure,
    })
}

pub fn log_error(error: &BridgeError) {
    console_error(&format!("Open Island bridge error: {error}"));
}

async fn invoke<T: serde::de::DeserializeOwned>(
    command: &str,
    args: &impl Serialize,
) -> Result<T, BridgeError> {
    let value = serde_wasm_bindgen::to_value(args).map_err(serde_error)?;
    let result = tauri_invoke(command, value).await.map_err(js_error)?;
    serde_wasm_bindgen::from_value(result).map_err(serde_error)
}

fn js_error(value: JsValue) -> BridgeError {
    BridgeError(
        value
            .as_string()
            .unwrap_or_else(|| "native bridge operation failed".into()),
    )
}

fn serde_error(error: impl std::fmt::Display) -> BridgeError {
    BridgeError(format!("invalid bridge payload: {error}"))
}

#[derive(Serialize)]
struct EmptyArgs {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivityArgs<'a> {
    activity_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionArgs<'a> {
    activity_id: &'a str,
    action_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnimationArgs {
    transition_id: u64,
    final_mode: IslandMode,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct DemoArgs {
    kind: ActivityKind,
    failed: Option<bool>,
}
