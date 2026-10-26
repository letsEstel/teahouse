// Teahouse's Android activity. CI copies this over the one `tauri android init`
// generates (src-tauri/gen/ is not committed), see .github/workflows/app.yml.
//
// Android 15 draws every app edge to edge: without this, the status bar's clock
// sits on top of the page. Rather than trusting env(safe-area-inset-*) (often 0
// in Android's WebView), the activity reads the window insets itself and pads
// the content view, so the page never knows the bars exist:
//
//   * padding = the largest of the status bar, navigation bar, display cutout and
//     on-screen keyboard on each side, re-applied whenever they change (a Pixel
//     Fold folding or unfolding, rotation, keyboard). The page then just sees a
//     smaller window and its own breakpoints pick the phone or tablet layout.
//   * the strips behind the bars are painted in the page's paper colour, light or
//     dark, and the bar icons switch to dark-on-light or light-on-dark to match.
//     The page reports its theme through `window.TeahouseAndroid.setDark(...)`;
//     until it does (and in "follow system" mode) the system night mode decides.
//
// "Take the whole teahouse away" writes into Download/Teahouse/ by path, which
// the media store does not notice: Files apps list the folders but no files.
// The page calls `window.TeahouseAndroid.scanFolder(path)` afterwards, and the
// media scanner registers every file under it (only under Download/Teahouse/).

package org.teahouse.app

import android.content.res.Configuration
import android.media.MediaScannerConnection
import android.os.Environment
import android.graphics.Color
import android.os.Build
import android.os.Bundle
import android.view.View
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import java.io.File

class MainActivity : TauriActivity() {
  // The two paper colours of app/src/styles/base.css (--paper, light and dark).
  private val paperLight = Color.parseColor("#F1ECE2")
  private val paperDark = Color.parseColor("#161514")

  /** What the page asked for; null = follow the system's night mode. */
  private var pageDark: Boolean? = null

  override fun onCreate(savedInstanceState: Bundle?) {
    // Draw behind the bars on every Android version, not only 15, so all
    // devices behave the same; the padding below keeps content clear of them.
    WindowCompat.setDecorFitsSystemWindows(window, false)
    @Suppress("DEPRECATION")
    if (Build.VERSION.SDK_INT < 35) {
      window.statusBarColor = Color.TRANSPARENT
      window.navigationBarColor = Color.TRANSPARENT
    }
    // No grey scrim over a three-button navigation bar: the paper shows through.
    if (Build.VERSION.SDK_INT >= 29) {
      window.isNavigationBarContrastEnforced = false
      window.isStatusBarContrastEnforced = false
    }
    super.onCreate(savedInstanceState)

    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val safe = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or
          WindowInsetsCompat.Type.displayCutout() or
          WindowInsetsCompat.Type.ime(),
      )
      view.setPadding(safe.left, safe.top, safe.right, safe.bottom)
      WindowInsetsCompat.CONSUMED
    }
    ViewCompat.requestApplyInsets(content)
    applyTheme()
  }

  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(Bridge(), "TeahouseAndroid")
    webView.setBackgroundColor(currentPaper())
    ViewCompat.requestApplyInsets(findViewById(android.R.id.content))
  }

  override fun onConfigurationChanged(newConfig: Configuration) {
    super.onConfigurationChanged(newConfig)
    // Night mode switched, or the Fold opened/closed: repaint and re-pad.
    applyTheme()
    ViewCompat.requestApplyInsets(findViewById(android.R.id.content))
  }

  private fun systemDark(): Boolean =
    (resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK) == Configuration.UI_MODE_NIGHT_YES

  private fun isDark(): Boolean = pageDark ?: systemDark()

  private fun currentPaper(): Int = if (isDark()) paperDark else paperLight

  private fun applyTheme() {
    val paper = currentPaper()
    window.decorView.setBackgroundColor(paper)
    findViewById<View>(android.R.id.content)?.setBackgroundColor(paper)
    WindowCompat.getInsetsController(window, window.decorView).apply {
      // "light" bars = dark icons, for the light paper
      isAppearanceLightStatusBars = !isDark()
      isAppearanceLightNavigationBars = !isDark()
    }
  }

  /** window.TeahouseAndroid in the page. */
  inner class Bridge {
    /** The page's effective theme: true = dark paper. */
    @JavascriptInterface
    fun setDark(dark: Boolean) {
      runOnUiThread {
        pageDark = dark
        applyTheme()
      }
    }

    /** Registers the files of an exported teahouse with the media store. */
    @JavascriptInterface
    fun scanFolder(path: String) {
      val root = File(Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS), "Teahouse").canonicalFile
      val dir = File(path).canonicalFile
      if (!dir.path.startsWith(root.path + File.separator)) return
      val files = dir.walkTopDown().filter { it.isFile }.map { it.path }.toList()
      if (files.isNotEmpty()) MediaScannerConnection.scanFile(applicationContext, files.toTypedArray(), null, null)
    }
  }
}
