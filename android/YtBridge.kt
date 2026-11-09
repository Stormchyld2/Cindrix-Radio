package app.cindrix.radio

/** Called from Rust. Touches no yt-dlp classes itself, so a failure inside the library
 *  comes back as readable text (with the whole cause chain) instead of a bare class name. */
object YtBridge {
  private fun <T> guard(f: () -> T): T {
    try {
      return f()
    } catch (t: Throwable) {
      val sb = StringBuilder("yt-dlp helper: ")
      var c: Throwable? = t
      var n = 0
      while (c != null && n < 6) {
        sb.append(c.toString())
        val top = c.stackTrace.firstOrNull()
        if (top != null) sb.append(" @ ").append(top.toString())
        sb.append(" | ")
        c = c.cause
        n++
      }
      throw RuntimeException(sb.toString())
    }
  }

  @JvmStatic fun version(): String = guard { YtImpl.version() }
  @JvmStatic fun update(): String = guard { YtImpl.update() }
  @JvmStatic fun search(target: String, single: Boolean): String = guard { YtImpl.search(target, single) }
  @JvmStatic fun download(url: String, dir: String): String = guard { YtImpl.download(url, dir) }
}
