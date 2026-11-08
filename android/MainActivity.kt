package app.cindrix.radio

import android.Manifest
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.Settings
import android.view.View
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    MediaBridge.init(this)
    // keep the app clear of the status bar, navigation bar and keyboard
    val root = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(root) { v, insets ->
      val b = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.ime())
      v.setPadding(b.left, b.top, b.right, b.bottom)
      insets
    }
    askPermissions()
    try {
      startForegroundService(Intent(this, PlaybackService::class.java))
    } catch (e: Exception) {
      // playback still works while the app is open
    }
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
    askAllFiles()
  }

  // The library scan and Downloads/Cindrix use ordinary file paths, which needs all-files access.
  private fun askAllFiles() {
    if (Build.VERSION.SDK_INT >= 30 && !Environment.isExternalStorageManager()) {
      try {
        startActivity(Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION, Uri.parse("package:$packageName")))
      } catch (e: Exception) {
        startActivity(Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION))
      }
    }
  }
}
