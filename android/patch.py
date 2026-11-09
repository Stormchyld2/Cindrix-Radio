"""Run after `tauri android init`: copies our Kotlin files in and adds permissions + the service."""
import pathlib, re, shutil, sys
gen = pathlib.Path("src-tauri/gen/android/app/src/main")
man = gen / "AndroidManifest.xml"
s = man.read_text()
perms = ["INTERNET", "ACCESS_NETWORK_STATE", "MANAGE_EXTERNAL_STORAGE", "READ_EXTERNAL_STORAGE", "WRITE_EXTERNAL_STORAGE",
         "READ_MEDIA_AUDIO", "READ_MEDIA_VIDEO", "POST_NOTIFICATIONS", "FOREGROUND_SERVICE", "FOREGROUND_SERVICE_MEDIA_PLAYBACK",
         "BLUETOOTH_CONNECT", "WAKE_LOCK"]
add = "".join(f'    <uses-permission android:name="android.permission.{p}" />\n' for p in perms
              if f"android.permission.{p}\"" not in s)
s = s.replace("<application", add + "    <application", 1)
if "usesCleartextTraffic" not in s:
    s = s.replace("<application", '<application android:usesCleartextTraffic="true"', 1)
# GateActivity becomes the launcher: it shows a crash report (if any) before opening the app
m = re.search(r'(android:name="\.MainActivity".*?)<intent-filter>.*?</intent-filter>', s, flags=re.S)
if m: s = s[:m.start()] + m.group(1) + s[m.end():]
gate = ('    <activity android:name=".GateActivity" android:exported="true" android:theme="@android:style/Theme.Translucent.NoTitleBar">\n'
        '      <intent-filter><action android:name="android.intent.action.MAIN" /><category android:name="android.intent.category.LAUNCHER" /></intent-filter>\n'
        '    </activity>\n')
s = s.replace("</application>", gate + "    </application>", 1)
svc = '    <service android:name=".PlaybackService" android:exported="false" android:foregroundServiceType="mediaPlayback" />\n'
s = s.replace("</application>", svc + "    </application>", 1)
man.write_text(s)
pkg = next(gen.glob("java/**/MainActivity.kt")).parent
for f in ("MainActivity.kt", "GateActivity.kt", "PlaybackService.kt", "MediaBridge.kt", "YtBridge.kt", "YtImpl.kt"):
    shutil.copy(pathlib.Path("android") / f, pkg / f)

# C++ runtime for the audio library (cpal/oboe): link is added in build.rs, the library itself is packaged here
import os
ndk = os.environ.get("NDK_HOME", "")
cxx = pathlib.Path(ndk) / "toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so"
dest = pathlib.Path("src-tauri/gen/android/app/src/main/jniLibs/arm64-v8a")
dest.mkdir(parents=True, exist_ok=True)
if not cxx.exists():
    raise SystemExit(f"libc++_shared.so not found at {cxx}")
shutil.copy(cxx, dest / "libc++_shared.so")

# Gradle: bundle the open-source yt-dlp library, and let it unpack its native files
g = next(pathlib.Path("src-tauri/gen/android/app").glob("build.gradle.kts"))
t = g.read_text()
t = re.sub(r"^android \{", "android {\n    packaging { jniLibs { useLegacyPackaging = true } }", t, count=1, flags=re.M)
# private sideloaded build: older, calmer Android behaviour; plain-http radio logos allowed; no R8 surprises
t = t.replace("targetSdk = 37", "targetSdk = 35")
t = t.replace('manifestPlaceholders["usesCleartextTraffic"] = "false"', 'manifestPlaceholders["usesCleartextTraffic"] = "true"')
t = t.replace("enable = true", "enable = false")
t += '\ndependencies {\n    implementation("io.github.junkfood02.youtubedl-android:library:0.18.1")\n    implementation("commons-io:commons-io:2.5")\n}\nconfigurations.all { resolutionStrategy { force("commons-io:commons-io:2.5") } }\n'
g.write_text(t)

# keep the classes Rust calls by name (JNI) and the yt-dlp library from being stripped in release builds
pg = next(pathlib.Path("src-tauri/gen/android/app").glob("proguard-rules.pro"), None)
rules = "\n-keep class app.cindrix.radio.** { *; }\n-keep class com.yausername.** { *; }\n-keep class com.fasterxml.** { *; }\n-dontwarn **\n"
if pg is not None: pg.write_text(pg.read_text() + rules)
else: pathlib.Path("src-tauri/gen/android/app/proguard-rules.pro").write_text(rules)
print(man.read_text()); print(t)
