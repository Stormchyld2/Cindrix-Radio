package app.cindrix.radio

import android.Manifest
import android.app.Activity
import android.app.ActivityManager
import android.app.AlertDialog
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.Settings
import android.view.View
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import java.io.File

class MainActivity : TauriActivity() {
  private fun crashFile(): File = File(getExternalFilesDir(null) ?: filesDir, "crash.txt")

  // Anything that crashes the app is written down and shown on the next start.
  private fun installCrashHandler() {
    val prev = Thread.getDefaultUncaughtExceptionHandler()
    Thread.setDefaultUncaughtExceptionHandler { t, e ->
      try { crashFile().writeText("Crash in thread " + t.name + ":\n" + android.util.Log.getStackTraceString(e)) } catch (_: Throwable) {}
      prev?.uncaughtException(t, e)
    }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    installCrashHandler()
    super.onCreate(savedInstanceState)
    try {
      // keep the app clear of the status bar, navigation bar and keyboard
      val root = findViewById<View>(android.R.id.content)
      ViewCompat.setOnApplyWindowInsetsListener(root) { v, insets ->
        val b = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.ime())
        v.setPadding(b.left, b.top, b.right, b.bottom)
        insets
      }
    } catch (_: Throwable) {}
    try { MediaBridge.init(this) } catch (_: Throwable) {}
    try { askPermissions() } catch (_: Throwable) {}
  }

  private fun askPermissions() {
    val want = ArrayList<String>()
    if (Build.VERSION.SDK_INT >= 33) want.add(Manifest.permission.POST_NOTIFICATIONS)
    if (Build.VERSION.SDK_INT >= 31) want.add(Manifest.permission.BLUETOOTH_CONNECT)
    if (Build.VERSION.SDK_INT < 30) {
      want.add(Manifest.permission.READ_EXTERNAL_STORAGE)
      want.add(Manifest.permission.WRITE_EXTERNAL_STORAGE)
    }
    if (want.isNotEmpty()) requestPermissions(want.toTypedArray(), 1) else askAllFiles()
  }

  override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
    super.onRequestPermissionsResult(requestCode, permissions, grantResults)
    try { askAllFiles() } catch (_: Throwable) {}
  }

  // The library scan and Downloads/Cindrix use ordinary file paths, which needs all-files access.
  private fun askAllFiles() {
    if (Build.VERSION.SDK_INT >= 30 && !Environment.isExternalStorageManager()) {
      try {
        startActivity(Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION, Uri.parse("package:$packageName")))
      } catch (e: Exception) {
        try { startActivity(Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION)) } catch (_: Exception) {}
      }
    }
  }
}
