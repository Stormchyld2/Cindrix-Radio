package app.cindrix.radio

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.media.MediaMetadata
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.Build
import android.provider.Settings

/** Lock screen, Bluetooth/headset buttons, car metadata, audio focus. The player lives in Rust. */
object MediaBridge {
  @JvmStatic external fun nativeCommand(cmd: String)

  lateinit var ctx: Context
  var session: MediaSession? = null
  private var wasPlaying = false
  private var lostFocus = false
  private var focusReq: AudioFocusRequest? = null

  // short log of audio-route events for the Player log (Settings > Advanced & Diagnostics)
  private val events = java.util.ArrayDeque<String>()
  private val clock = java.text.SimpleDateFormat("HH:mm:ss", java.util.Locale.US)
  fun note(m: String) { synchronized(events) { events.addLast(clock.format(java.util.Date()) + " " + m); while (events.size > 60) events.removeFirst() } }

  private fun devs(list: Array<out android.media.AudioDeviceInfo>?): String =
    list?.joinToString(", ") { "type ${it.type} '${it.productName}'" } ?: "?"

  @JvmStatic
  fun dumpLog(): String {
    val sb = StringBuilder()
    try {
      val am = ctx.getSystemService(AudioManager::class.java)
      sb.append("now: mode ${am.mode}, A2DP ${am.isBluetoothA2dpOn}, SCO ${am.isBluetoothScoOn}, speaker ${am.isSpeakerphoneOn}, output rate ${am.getProperty(AudioManager.PROPERTY_OUTPUT_SAMPLE_RATE)}, frames/buffer ${am.getProperty(AudioManager.PROPERTY_OUTPUT_FRAMES_PER_BUFFER)}\n")
      sb.append("outputs: ").append(devs(am.getDevices(AudioManager.GET_DEVICES_OUTPUTS))).append("\n")
    } catch (t: Throwable) { sb.append("route info failed: $t\n") }
    synchronized(events) { events.forEach { sb.append(it).append("\n") } }
    return sb.toString()
  }

  private val focusListener = AudioManager.OnAudioFocusChangeListener { change ->
    note("audio focus change $change")
    when (change) {
      AudioManager.AUDIOFOCUS_LOSS -> { lostFocus = false; nativeCommand("pause") }
      AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> if (wasPlaying) { lostFocus = true; nativeCommand("pause") }
      AudioManager.AUDIOFOCUS_GAIN -> if (lostFocus) { lostFocus = false; nativeCommand("play") }
    }
  }

  private val noisy = object : BroadcastReceiver() {
    override fun onReceive(c: Context?, i: Intent?) { note("becoming noisy"); nativeCommand("noisy") }
  }

  @JvmStatic
  fun init(c: Context) {
    ctx = c.applicationContext
    if (session != null) return
    val s = MediaSession(ctx, "CindrixRadio")
    s.setCallback(object : MediaSession.Callback() {
      override fun onPlay() { nativeCommand("play") }
      override fun onPause() { nativeCommand("pause") }
      override fun onSkipToNext() { nativeCommand("next") }
      override fun onSkipToPrevious() { nativeCommand("prev") }
      override fun onStop() { nativeCommand("pause") }
    })
    s.isActive = true
    session = s
    val f = IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY)
    if (Build.VERSION.SDK_INT >= 33) ctx.registerReceiver(noisy, f, Context.RECEIVER_EXPORTED) else ctx.registerReceiver(noisy, f)
    // Bluetooth / headset connected or removed: tell the player to reopen its audio output
    val am = ctx.getSystemService(AudioManager::class.java)
    val h = android.os.Handler(android.os.Looper.getMainLooper())
    val cb = object : android.media.AudioDeviceCallback() {
      override fun onAudioDevicesAdded(d: Array<out android.media.AudioDeviceInfo>?) { note("device added: ${devs(d)}"); h.postDelayed({ nativeCommand("reroute") }, 900) }
      override fun onAudioDevicesRemoved(d: Array<out android.media.AudioDeviceInfo>?) { note("device removed: ${devs(d)}"); h.postDelayed({ nativeCommand("reroute") }, 900) }
    }
    am.registerAudioDeviceCallback(cb, h)
  }

  @JvmStatic
  fun update(title: String, subtitle: String, playing: Boolean, pos: Long, dur: Long) {
    val s = session ?: return
    val md = MediaMetadata.Builder()
      .putString(MediaMetadata.METADATA_KEY_TITLE, title)
      .putString(MediaMetadata.METADATA_KEY_ARTIST, subtitle)
    if (dur > 0) md.putLong(MediaMetadata.METADATA_KEY_DURATION, dur * 1000)
    s.setMetadata(md.build())
    val actions = PlaybackState.ACTION_PLAY or PlaybackState.ACTION_PAUSE or PlaybackState.ACTION_PLAY_PAUSE or
      PlaybackState.ACTION_SKIP_TO_NEXT or PlaybackState.ACTION_SKIP_TO_PREVIOUS or PlaybackState.ACTION_STOP
    s.setPlaybackState(
      PlaybackState.Builder().setActions(actions)
        .setState(if (playing) PlaybackState.STATE_PLAYING else PlaybackState.STATE_PAUSED, pos * 1000, if (playing) 1f else 0f)
        .build()
    )
    if (playing && !wasPlaying) requestFocus()
    if (!playing && wasPlaying && !lostFocus) abandonFocus()
    wasPlaying = playing
    if (playing && !PlaybackService.running) {
      try { ctx.startForegroundService(Intent(ctx, PlaybackService::class.java)) } catch (e: Throwable) {}
    }
    PlaybackService.refresh(ctx, title, subtitle, playing)
  }

  private fun requestFocus() {
    val am = ctx.getSystemService(AudioManager::class.java)
    if (Build.VERSION.SDK_INT >= 26) {
      val r = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
        .setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA).setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build())
        .setOnAudioFocusChangeListener(focusListener).build()
      focusReq = r
      am.requestAudioFocus(r)
    } else {
      @Suppress("DEPRECATION")
      am.requestAudioFocus(focusListener, AudioManager.STREAM_MUSIC, AudioManager.AUDIOFOCUS_GAIN)
    }
  }

  private fun abandonFocus() {
    val am = ctx.getSystemService(AudioManager::class.java)
    if (Build.VERSION.SDK_INT >= 26) focusReq?.let { am.abandonAudioFocusRequest(it) }
    else @Suppress("DEPRECATION") am.abandonAudioFocus(focusListener)
  }

  @JvmStatic
  fun openBluetooth() {
    ctx.startActivity(Intent(Settings.ACTION_BLUETOOTH_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
  }
}
