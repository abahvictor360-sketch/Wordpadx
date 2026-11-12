package com.wordpadx.app

import android.annotation.SuppressLint
import android.app.DownloadManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.OpenableColumns
import android.util.Base64
import android.util.Log
import android.view.View
import android.view.WindowManager
import android.webkit.*
import android.webkit.MimeTypeMap
import android.widget.Toast
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.webkit.WebViewAssetLoader
import java.io.File
import java.io.FileOutputStream

class MainActivity : AppCompatActivity() {

    private lateinit var webView: WebView
    private var filePathCallback: ValueCallback<Array<Uri>>? = null

    /// A document opened from another app ("Open with Wordpadx", Share) waiting for the page.
    private var pendingDocument: Uri? = null
    private var pageLoaded = false

    private val fileChooserLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (filePathCallback != null) {
            val results = if (result.resultCode == RESULT_OK && result.data != null) {
                val data = result.data
                if (data?.clipData != null) {
                    val count = data.clipData!!.itemCount
                    Array(count) { i -> data.clipData!!.getItemAt(i).uri }
                } else if (data?.data != null) {
                    arrayOf(data.data!!)
                } else {
                    null
                }
            } else {
                null
            }
            filePathCallback?.onReceiveValue(results)
            filePathCallback = null
        }
    }

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Tablet-friendly edge-to-edge / fullscreen configuration
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            window.attributes.layoutInDisplayCutoutMode =
                WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
        }

        webView = WebView(this)
        webView.isFocusable = true
        webView.isFocusableInTouchMode = true
        webView.requestFocus(View.FOCUS_DOWN)
        setContentView(webView)
        window.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)

        configureWebView()
        setupAssetLoaderAndLoad()
        pendingDocument = documentUri(intent)
    }

    /// Opened again while running (singleTask): open the new document in the running app.
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        documentUri(intent)?.let {
            pendingDocument = it
            if (pageLoaded) deliverPendingDocument()
        }
    }

    /// The document an "Open with" (ACTION_VIEW) or Share (ACTION_SEND) intent carries.
    private fun documentUri(intent: Intent?): Uri? = when (intent?.action) {
        Intent.ACTION_VIEW -> intent.data
        Intent.ACTION_SEND -> if (Build.VERSION.SDK_INT >= 33) {
            intent.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)
        } else {
            @Suppress("DEPRECATION")
            intent.getParcelableExtra(Intent.EXTRA_STREAM)
        }
        else -> null
    }

    /// The file's name as the providing app reports it, or one made from its type.
    private fun displayName(uri: Uri): String {
        try {
            contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { c ->
                if (c.moveToFirst()) {
                    val i = c.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    if (i >= 0) c.getString(i)?.takeIf { it.isNotBlank() }?.let { return it }
                }
            }
        } catch (e: Exception) {
            Log.w("Wordpadx", "No display name for $uri", e)
        }
        uri.lastPathSegment?.substringAfterLast('/')?.takeIf { it.contains('.') }?.let { return it }
        val ext = contentResolver.getType(uri)?.let { MimeTypeMap.getSingleton().getExtensionFromMimeType(it) } ?: "docx"
        return "document.$ext"
    }

    /// Read the pending document and hand it to the web app (`wordpadxOpenFile`), retrying until
    /// the app has started; the page loads the app asynchronously after `onPageFinished`.
    private fun deliverPendingDocument() {
        val uri = pendingDocument ?: return
        pendingDocument = null
        Thread {
            try {
                val bytes = contentResolver.openInputStream(uri)?.use { input ->
                    val out = java.io.ByteArrayOutputStream()
                    val buf = ByteArray(64 * 1024)
                    var total = 0
                    while (true) {
                        val n = input.read(buf)
                        if (n < 0) break
                        total += n
                        if (total > MAX_DOCUMENT_BYTES) throw java.io.IOException("larger than 200 MB")
                        out.write(buf, 0, n)
                    }
                    out.toByteArray()
                } ?: throw java.io.IOException("the file couldn't be read")
                val name = displayName(uri)
                val b64 = Base64.encodeToString(bytes, Base64.NO_WRAP)
                val js = "(function o(t){if(window.wordpadxOpenFile){wordpadxOpenFile(" +
                    org.json.JSONObject.quote(name) + ",'" + b64 + "')}else if(t<300){setTimeout(function(){o(t+1)},200)}})(0)"
                runOnUiThread { webView.evaluateJavascript(js, null) }
            } catch (e: Exception) {
                Log.e("Wordpadx", "Couldn't open $uri", e)
                runOnUiThread { Toast.makeText(this, "Couldn't open the document: ${e.message}", Toast.LENGTH_LONG).show() }
            }
        }.start()
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun configureWebView() {
        val settings = webView.settings
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.databaseEnabled = true
        settings.allowFileAccess = true
        settings.allowContentAccess = true
        settings.loadWithOverviewMode = true
        settings.useWideViewPort = true
        settings.builtInZoomControls = false
        settings.displayZoomControls = false
        settings.mediaPlaybackRequiresUserGesture = false
        settings.setNeedInitialFocus(true)
        settings.cacheMode = WebSettings.LOAD_DEFAULT

        // Optimize for tablet GPU rendering
        webView.setLayerType(View.LAYER_TYPE_HARDWARE, null)

        webView.webChromeClient = object : WebChromeClient() {
            override fun onShowFileChooser(
                webView: WebView?,
                filePathCallback: ValueCallback<Array<Uri>>?,
                fileChooserParams: FileChooserParams?
            ): Boolean {
                this@MainActivity.filePathCallback?.onReceiveValue(null)
                this@MainActivity.filePathCallback = filePathCallback

                // The page asks for extensions (".docx"); Android's picker wants MIME types, and
                // greys out files it can't match, so map them and let anything else through.
                val mimes = fileChooserParams?.acceptTypes.orEmpty()
                    .flatMap { it.split(',') }
                    .map { it.trim().lowercase() }
                    .filter { it.isNotEmpty() }
                    .mapNotNull { t -> if (t.startsWith(".")) MimeTypeMap.getSingleton().getMimeTypeFromExtension(t.drop(1)) else t }
                    .distinct()
                val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
                    type = "*/*"
                    addCategory(Intent.CATEGORY_OPENABLE)
                    if (mimes.isNotEmpty()) putExtra(Intent.EXTRA_MIME_TYPES, (mimes + "application/octet-stream").toTypedArray())
                }

                try {
                    fileChooserLauncher.launch(intent)
                } catch (e: Exception) {
                    Log.e("Wordpadx", "Failed to launch file chooser", e)
                    this@MainActivity.filePathCallback = null
                    return false
                }
                return true
            }

            override fun onConsoleMessage(consoleMessage: ConsoleMessage?): Boolean {
                consoleMessage?.let {
                    Log.d("Wordpadx-Web", "${it.message()} -- From line ${it.lineNumber()} of ${it.sourceId()}")
                }
                return true
            }
        }

        webView.setDownloadListener { url, _, contentDisposition, mimeType, _ ->
            handleDownload(url, contentDisposition, mimeType)
        }
    }

    private fun setupAssetLoaderAndLoad() {
        val assetLoader = WebViewAssetLoader.Builder()
            .addPathHandler("/assets/", WebViewAssetLoader.AssetsPathHandler(this))
            .build()

        webView.webViewClient = object : WebViewClient() {
            override fun shouldInterceptRequest(
                view: WebView?,
                request: WebResourceRequest?
            ): WebResourceResponse? {
                val uri = request?.url ?: return null
                return assetLoader.shouldInterceptRequest(uri)
            }

            override fun onPageFinished(view: WebView?, url: String?) {
                super.onPageFinished(view, url)
                pageLoaded = true
                deliverPendingDocument()
            }

            override fun onReceivedError(
                view: WebView?,
                request: WebResourceRequest?,
                error: WebResourceError?
            ) {
                super.onReceivedError(view, request, error)
                if (request?.isForMainFrame == true) {
                    Log.e("Wordpadx", "Page load error: ${error?.description}")
                }
            }
        }

        // Check if local compiled web assets exist in assets folder
        val hasLocalAssets = try {
            assets.list("www")?.isNotEmpty() == true || assets.list("")?.contains("index.html") == true
        } catch (e: Exception) {
            false
        }

        if (hasLocalAssets) {
            // Secure virtual domain mapped to Android assets for WebGPU / Wasm / Clipboard support
            webView.loadUrl("https://appassets.androidplatform.net/assets/www/index.html")
        } else {
            // Default fallback or development load
            webView.loadUrl("https://appassets.androidplatform.net/assets/index.html")
        }
    }

    private fun handleDownload(url: String, contentDisposition: String, mimeType: String) {
        if (url.startsWith("data:")) {
            saveDataUri(url, contentDisposition)
            return
        }

        try {
            val request = DownloadManager.Request(Uri.parse(url)).apply {
                setMimeType(mimeType)
                setDescription("Downloading Wordpadx document...")
                val filename = URLUtil.guessFileName(url, contentDisposition, mimeType)
                setTitle(filename)
                setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
                setDestinationInExternalPublicDir(Environment.DIRECTORY_DOWNLOADS, filename)
            }

            val dm = getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager
            dm.enqueue(request)
            Toast.makeText(this, "Downloading document to Downloads folder...", Toast.LENGTH_SHORT).show()
        } catch (e: Exception) {
            Log.e("Wordpadx", "Download error", e)
            Toast.makeText(this, "Download failed: ${e.message}", Toast.LENGTH_LONG).show()
        }
    }

    private fun saveDataUri(dataUrl: String, contentDisposition: String) {
        try {
            val commaIndex = dataUrl.indexOf(',')
            if (commaIndex == -1) return
            val metadata = dataUrl.substring(0, commaIndex)
            val base64Data = dataUrl.substring(commaIndex + 1)
            val bytes = Base64.decode(base64Data, Base64.DEFAULT)

            val ext = when {
                metadata.contains("pdf") -> ".pdf"
                metadata.contains("wordprocessingml") -> ".docx"
                metadata.contains("opendocument") -> ".odt"
                else -> ".bin"
            }
            val filename = "Wordpadx_Export_${System.currentTimeMillis()}$ext"
            val downloadsDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
            val file = File(downloadsDir, filename)
            FileOutputStream(file).use { it.write(bytes) }
            Toast.makeText(this, "Saved to Downloads: $filename", Toast.LENGTH_LONG).show()
        } catch (e: Exception) {
            Log.e("Wordpadx", "Failed saving data URI", e)
        }
    }

    companion object {
        private const val MAX_DOCUMENT_BYTES = 200 * 1024 * 1024
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {
        if (webView.canGoBack()) {
            webView.goBack()
        } else {
            super.onBackPressed()
        }
    }
}
