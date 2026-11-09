//! Bridge between the Rust player and Android: lock-screen / Bluetooth / headset controls
//! (MediaSession), audio focus, and the bundled yt-dlp library. The Kotlin side lives in
//! android/MediaBridge.kt and android/YtBridge.kt (copied into the Android project by CI).
use jni::objects::{JClass, JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};
use std::sync::{Mutex, OnceLock};
use tauri::AppHandle;

use crate::player::PlayerState;

static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app(h: AppHandle) { let _ = APP.set(h); }

fn exception_text(env: &mut JNIEnv) -> String {
    if env.exception_check().unwrap_or(false) {
        let ex = env.exception_occurred();
        let _ = env.exception_clear();
        if let Ok(ex) = ex {
            if let Ok(v) = env.call_method(&ex, "toString", "()Ljava/lang/String;", &[]) {
                if let Ok(o) = v.l() {
                    if let Ok(s) = env.get_string(&JString::from(o)) { return s.into(); }
                }
            }
        }
        return "Android reported an error".into();
    }
    "Android call failed".into()
}

/// Run `f` with a JNI environment attached to this thread and the app's class loader.
fn with_env<T>(f: impl FnOnce(&mut JNIEnv, &JObject) -> Result<T, String>) -> Result<T, String> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| e.to_string())?;
    let mut guard = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
    f(&mut guard, &activity)
}

fn class<'a>(env: &mut JNIEnv<'a>, ctx: &JObject, name: &str) -> Result<JClass<'a>, String> {
    let loader = env.call_method(ctx, "getClassLoader", "()Ljava/lang/ClassLoader;", &[]).and_then(|v| v.l())
        .map_err(|_| exception_text(env))?;
    let jname = env.new_string(name).map_err(|e| e.to_string())?;
    let cls = env.call_method(&loader, "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;", &[JValue::Object(&jname)])
        .and_then(|v| v.l()).map_err(|_| exception_text(env))?;
    Ok(JClass::from(cls))
}

fn call_void(cls: &str, method: &str, sig: &str, args: &[JValue]) -> Result<(), String> {
    with_env(|env, ctx| {
        let c = class(env, ctx, cls)?;
        env.call_static_method(&c, method, sig, args).map(|_| ()).map_err(|_| exception_text(env))
    })
}

fn call_string(cls: &str, method: &str, sig: &str, args: &[JValue]) -> Result<String, String> {
    with_env(|env, ctx| {
        let c = class(env, ctx, cls)?;
        let v = env.call_static_method(&c, method, sig, args).map_err(|_| exception_text(env))?;
        let o = v.l().map_err(|e| e.to_string())?;
        if o.is_null() { return Ok(String::new()); }
        env.get_string(&JString::from(o)).map(|s| s.into()).map_err(|e| e.to_string())
    })
}

const MEDIA: &str = "app.cindrix.radio.MediaBridge";
const YT: &str = "app.cindrix.radio.YtBridge";
const S: &str = "Ljava/lang/String;";

// ---- yt-dlp (Android library) ----
pub fn yt_version() -> Result<String, String> { call_string(YT, "version", &format!("(){S}"), &[]) }
pub fn yt_update() -> Result<String, String> { call_string(YT, "update", &format!("(){S}"), &[]) }
pub fn yt_search(target: &str, single: bool) -> Result<String, String> {
    with_env(|env, ctx| {
        let c = class(env, ctx, YT)?;
        let t = env.new_string(target).map_err(|e| e.to_string())?;
        let v = env.call_static_method(&c, "search", &format!("({S}Z){S}"), &[JValue::Object(&t), JValue::Bool(single as u8)])
            .map_err(|_| exception_text(env))?;
        let o = v.l().map_err(|e| e.to_string())?;
        env.get_string(&JString::from(o)).map(|s| s.into()).map_err(|e| e.to_string())
    })
}
pub fn yt_download(url: &str, dir: &str) -> Result<String, String> {
    with_env(|env, ctx| {
        let c = class(env, ctx, YT)?;
        let u = env.new_string(url).map_err(|e| e.to_string())?;
        let d = env.new_string(dir).map_err(|e| e.to_string())?;
        let v = env.call_static_method(&c, "download", &format!("({S}{S}){S}"), &[JValue::Object(&u), JValue::Object(&d)])
            .map_err(|_| exception_text(env))?;
        let o = v.l().map_err(|e| e.to_string())?;
        env.get_string(&JString::from(o)).map(|s| s.into()).map_err(|e| e.to_string())
    })
}

// ---- lock screen / Bluetooth / notification ----
static LAST: Mutex<(String, bool, u64)> = Mutex::new((String::new(), false, 0));

/// Called from the 500 ms player poll. Pushes changes (track, play/pause, a position jump or
/// every 10 s) to the MediaSession so the lock screen, car and headset show the right thing.
/// Audio route and focus events recorded on the Kotlin side, for the Player log.
pub fn media_log() -> String { call_string(MEDIA, "dumpLog", &format!("(){S}"), &[]).unwrap_or_default() }

pub fn media_sync(s: &PlayerState) {
    let Some(t) = s.current.as_ref() else {
        let mut l = LAST.lock().unwrap();
        if !l.0.is_empty() {
            *l = (String::new(), false, 0);
            drop(l);
            let _ = push("Cindrix Radio", "Nothing playing", false, 0, 0);
        }
        return;
    };
    let title = s.now_playing.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| t.title.clone());
    let key = format!("{}|{}", t.id, title);
    let pos = s.position_secs as u64;
    let mut l = LAST.lock().unwrap();
    let changed = l.0 != key || l.1 != s.playing;
    let jumped = pos.abs_diff(l.2) >= 10;
    if !changed && !jumped { return; }
    *l = (key, s.playing, pos);
    drop(l);
    let sub = if s.now_playing.is_some() { t.title.clone() } else { t.subtitle.clone() };
    let _ = push(&title, &sub, s.playing, pos, t.duration_secs.unwrap_or(0) as u64);
}

fn push(title: &str, sub: &str, playing: bool, pos: u64, dur: u64) -> Result<(), String> {
    with_env(|env, ctx| {
        let c = class(env, ctx, MEDIA)?;
        let a = env.new_string(title).map_err(|e| e.to_string())?;
        let b = env.new_string(sub).map_err(|e| e.to_string())?;
        env.call_static_method(&c, "update", &format!("({S}{S}ZJJ)V"),
            &[JValue::Object(&a), JValue::Object(&b), JValue::Bool(playing as u8), JValue::Long(pos as i64), JValue::Long(dur as i64)])
            .map(|_| ()).map_err(|_| exception_text(env))
    })
}

pub fn open_bluetooth_settings() -> Result<(), String> { call_void(MEDIA, "openBluetooth", "()V", &[]) }

/// Kotlin -> Rust: a lock-screen, headset, notification or audio-focus command.
#[no_mangle]
pub extern "system" fn Java_app_cindrix_radio_MediaBridge_nativeCommand(mut env: JNIEnv, _class: JClass, cmd: JString) {
    let cmd: String = env.get_string(&cmd).map(|s| s.into()).unwrap_or_default();
    if let Some(app) = APP.get() { crate::media_command(app, &cmd); }
}
