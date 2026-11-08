package app.cindrix.radio

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.graphics.drawable.Icon
import android.os.Build
import android.os.IBinder

// Foreground service: keeps the app alive with the screen off and shows the media notification.
class PlaybackService : Service() {
  companion object {
    var title = "Cindrix Radio"
    var subtitle = ""
    var playing = false
    var instance: PlaybackService? = null

    fun refresh(ctx: Context, t: String, s: String, p: Boolean) {
      title = t; subtitle = s; playing = p
      instance?.post()
    }
  }

  override fun onBind(intent: Intent?): IBinder? = null

  override fun onCreate() {
    super.onCreate()
    instance = this
    getSystemService(NotificationManager::class.java)
      .createNotificationChannel(NotificationChannel("playback", "Playback", NotificationManager.IMPORTANCE_LOW))
  }

  override fun onDestroy() {
    instance = null
    super.onDestroy()
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    when (intent?.action) {
      "prev", "toggle", "next" -> try { MediaBridge.nativeCommand(intent.action!!) } catch (e: Throwable) {}
    }
    post()
    return START_STICKY
  }

  private fun action(name: String, icon: Int, label: String): Notification.Action {
    val pi = PendingIntent.getService(this, name.hashCode(), Intent(this, PlaybackService::class.java).setAction(name), PendingIntent.FLAG_IMMUTABLE)
    return Notification.Action.Builder(Icon.createWithResource(this, icon), label, pi).build()
  }

  fun post() {
    val open = PendingIntent.getActivity(this, 0, Intent(this, MainActivity::class.java), PendingIntent.FLAG_IMMUTABLE)
    val ic = if (applicationInfo.icon != 0) applicationInfo.icon else android.R.drawable.ic_media_play
    val style = Notification.MediaStyle().setShowActionsInCompactView(0, 1, 2)
    MediaBridge.session?.let { style.setMediaSession(it.sessionToken) }
    val n = Notification.Builder(this, "playback")
      .setContentTitle(title)
      .setContentText(subtitle)
      .setSmallIcon(ic)
      .setContentIntent(open)
      .setOngoing(playing)
      .setStyle(style)
      .addAction(action("prev", android.R.drawable.ic_media_previous, "Previous"))
      .addAction(action("toggle", if (playing) android.R.drawable.ic_media_pause else android.R.drawable.ic_media_play, if (playing) "Pause" else "Play"))
      .addAction(action("next", android.R.drawable.ic_media_next, "Next"))
      .build()
    if (Build.VERSION.SDK_INT >= 29) startForeground(1, n, ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK)
    else startForeground(1, n)
  }
}
