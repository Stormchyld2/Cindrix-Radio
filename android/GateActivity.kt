package app.cindrix.radio

import android.app.Activity
import android.app.ActivityManager
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.graphics.Typeface
import android.os.Build
import android.os.Bundle
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import java.io.File

/**
 * The launcher entry. Normally it opens the app straight away. If the app crashed last time it
 * shows why first (so a crash at start-up can be read instead of looping), then opens the app.
 */
class GateActivity : Activity() {
  private fun launch() {
    startActivity(Intent(this, MainActivity::class.java))
    finish()
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    val prefs = getSharedPreferences("gate", Context.MODE_PRIVATE)
    var ts = 0L
    val sb = StringBuilder()
    try {
      if (Build.VERSION.SDK_INT >= 30) {
        val info = getSystemService(ActivityManager::class.java).getHistoricalProcessExitReasons(packageName, 0, 1).firstOrNull()
        if (info != null && info.reason in listOf(2, 4, 5, 6, 7) && info.timestamp > prefs.getLong("seen", 0)) {
          ts = info.timestamp
          sb.append("Android says: exit reason ").append(info.reason).append(" (").append(info.description).append(")\n\n")
        }
      }
      for (name in listOf("crash.txt", "rust-crash.txt")) {
        for (dir in listOfNotNull(getExternalFilesDir(null), filesDir)) {
          val f = File(dir, name)
          if (f.exists()) { sb.append(f.readText().take(2500)).append("\n\n"); f.delete() }
        }
      }
      if (sb.isNotEmpty()) {
        try {
          val p = Runtime.getRuntime().exec(arrayOf("logcat", "-d", "-t", "3000", "-v", "threadtime"))
          val keep = Regex("(?i)fatal|exception|signal|abort|panic|sigsegv|backtrace|cindrix|rust|oboe|tauri|wry|libc|dlopen|unsatisfied|androidruntime|tombstone")
          val lines = p.inputStream.bufferedReader().readLines().filter { keep.containsMatchIn(it) }.takeLast(70)
          sb.append("--- log ---\n").append(lines.joinToString("\n") { it.take(260) })
        } catch (_: Throwable) {}
      }
    } catch (_: Throwable) {}

    if (sb.isEmpty()) { launch(); return }
    val report = sb.toString().take(9000)

    val root = LinearLayout(this)
    root.orientation = LinearLayout.VERTICAL
    root.setBackgroundColor(Color.parseColor("#0A0E13"))
    root.setPadding(36, 90, 36, 36)
    val title = TextView(this)
    title.text = "Cindrix Radio closed unexpectedly last time"
    title.setTextColor(Color.WHITE)
    title.textSize = 18f
    root.addView(title)
    val body = TextView(this)
    body.text = report
    body.setTextColor(Color.parseColor("#C8D0D9"))
    body.textSize = 10f
    body.typeface = Typeface.MONOSPACE
    body.setTextIsSelectable(true)
    val scroll = ScrollView(this)
    scroll.addView(body)
    root.addView(scroll, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
    val copy = Button(this)
    copy.text = "Copy report"
    copy.setOnClickListener {
      getSystemService(ClipboardManager::class.java).setPrimaryClip(ClipData.newPlainText("Cindrix Radio crash", report))
    }
    root.addView(copy)
    val go = Button(this)
    go.text = "Open Cindrix Radio anyway"
    go.setOnClickListener {
      prefs.edit().putLong("seen", if (ts > 0) ts else System.currentTimeMillis()).apply()
      launch()
    }
    root.addView(go)
    setContentView(root)
  }
}
