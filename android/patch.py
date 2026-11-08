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
svc = '    <service android:name=".PlaybackService" android:exported="false" android:foregroundServiceType="mediaPlayback" />\n'
s = s.replace("</application>", svc + "    </application>", 1)
man.write_text(s)
pkg = next(gen.glob("java/**/MainActivity.kt")).parent
for f in ("MainActivity.kt", "PlaybackService.kt", "MediaBridge.kt", "YtBridge.kt"):
    shutil.copy(pathlib.Path("android") / f, pkg / f)

# Gradle: bundle the open-source yt-dlp library, and let it unpack its native files
g = next(pathlib.Path("src-tauri/gen/android/app").glob("build.gradle.kts"))
t = g.read_text()
t = re.sub(r"^android \{", "android {\n    packaging { jniLibs { useLegacyPackaging = true } }", t, count=1, flags=re.M)
t += '\ndependencies {\n    implementation("io.github.junkfood02.youtubedl-android:library:0.18.1")\n}\n'
g.write_text(t)
print(man.read_text()); print(t)
