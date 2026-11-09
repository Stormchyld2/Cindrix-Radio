package app.cindrix.radio

import com.yausername.youtubedl_android.YoutubeDL
import com.yausername.youtubedl_android.YoutubeDLRequest

/** The open-source yt-dlp Android library (private build only). Called from Rust. */
object YtBridge {
  private var ready = false

  @Synchronized
  private fun ensure() {
    if (!ready) {
      try {
        YoutubeDL.getInstance().init(MediaBridge.ctx)
      } catch (t: Throwable) {
        val sb = StringBuilder("yt-dlp helper failed to start: ")
        var c: Throwable? = t
        var n = 0
        while (c != null && n < 5) {
          sb.append(c.toString()).append(" @ ").append(c.stackTrace.firstOrNull()?.toString() ?: "").append(" | ")
          c = c.cause; n++
        }
        throw RuntimeException(sb.toString())
      }
      ready = true
    }
  }

  @JvmStatic
  fun version(): String {
    ensure()
    return YoutubeDL.version(MediaBridge.ctx) ?: ""
  }

  @JvmStatic
  fun update(): String {
    ensure()
    YoutubeDL.getInstance().updateYoutubeDL(MediaBridge.ctx, YoutubeDL.UpdateChannel.STABLE)
    return version()
  }

  @JvmStatic
  fun search(target: String, single: Boolean): String {
    ensure()
    val r = YoutubeDLRequest(target)
    r.addOption("--dump-single-json")
    r.addOption("--flat-playlist")
    r.addOption("--no-warnings")
    if (single) r.addOption("--no-playlist")
    return YoutubeDL.getInstance().execute(r).out
  }

  @JvmStatic
  fun download(url: String, dir: String): String {
    ensure()
    java.io.File(dir).mkdirs()
    val r = YoutubeDLRequest(url)
    r.addOption("-f", "b[ext=mp4]/b")
    r.addOption("--no-playlist")
    r.addOption("--no-warnings")
    r.addOption("--windows-filenames")
    r.addOption("--no-mtime")
    r.addOption("-q")
    r.addOption("--no-simulate")
    r.addOption("--print", "after_move:filepath")
    r.addOption("-o", "$dir/%(title).150B.%(ext)s")
    val out = YoutubeDL.getInstance().execute(r).out
    return out.lines().lastOrNull { it.isNotBlank() }?.trim() ?: ""
  }
}
