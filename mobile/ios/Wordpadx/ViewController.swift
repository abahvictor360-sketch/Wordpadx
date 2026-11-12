import UIKit
import WebKit

class ViewController: UIViewController, WKNavigationDelegate, WKUIDelegate, UIDocumentPickerDelegate {
    private var webView: WKWebView!
    /// Documents opened before the page finished loading, handed over once it has.
    private var pendingDocuments: [(name: String, base64: String)] = []
    private var pageLoaded = false
    private static let maxDocumentBytes = 200 * 1024 * 1024

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = UIColor(red: 0.17, green: 0.28, blue: 0.71, alpha: 1.0) // Wordpadx brand

        setupWebView()
        loadApp()
    }

    private func setupWebView() {
        let preferences = WKPreferences()
        let config = WKWebViewConfiguration()
        config.preferences = preferences
        config.allowsInlineMediaPlayback = true

        // Configure WKUserContentController for tablet gestures
        let controller = WKUserContentController()
        config.userContentController = controller

        webView = WKWebView(frame: .zero, configuration: config)
        webView.translatesAutoresizingMaskIntoConstraints = false
        webView.navigationDelegate = self
        webView.uiDelegate = self
        webView.isUserInteractionEnabled = true
        webView.scrollView.keyboardDismissMode = .none
        webView.scrollView.isScrollEnabled = false // Let egui canvas handle its own scrolling
        webView.scrollView.bounces = false
        webView.isOpaque = false
        webView.backgroundColor = .clear

        view.addSubview(webView)

        NSLayoutConstraint.activate([
            webView.topAnchor.constraint(equalTo: view.topAnchor),
            webView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            webView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            webView.trailingAnchor.constraint(equalTo: view.trailingAnchor)
        ])
    }

    private func loadApp() {
        if let bundlePath = Bundle.main.url(forResource: "index", withExtension: "html", subdirectory: "www") {
            let directoryPath = bundlePath.deletingLastPathComponent()
            webView.loadFileURL(bundlePath, allowingReadAccessTo: directoryPath)
        } else if let bundlePath = Bundle.main.url(forResource: "index", withExtension: "html") {
            let directoryPath = bundlePath.deletingLastPathComponent()
            webView.loadFileURL(bundlePath, allowingReadAccessTo: directoryPath)
        } else {
            // Fallback for development / remote loading
            let html = """
            <!DOCTYPE html>
            <html>
            <head><meta name="viewport" content="width=device-width, initial-scale=1.0"></head>
            <body style="font-family: -apple-system; text-align: center; padding-top: 50px; background: #e6e6e6;">
                <h2>Wordpadx for iPad</h2>
                <p>Compile web assets to <code>mobile/ios/Wordpadx/www/</code> to run offline.</p>
            </body>
            </html>
            """
            webView.loadHTMLString(html, baseURL: nil)
        }
    }

    // Handle iPad document downloads and file sharing
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        if let url = navigationAction.request.url {
            if url.scheme == "blob" || url.pathExtension == "docx" || url.pathExtension == "pdf" {
                // Allow action
                decisionHandler(.allow)
                return
            }
        }
        decisionHandler(.allow)
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        pageLoaded = true
        let pending = pendingDocuments
        pendingDocuments.removeAll()
        pending.forEach(deliver)
    }

    /// Open a document another app handed over: read it (with security-scoped access when it
    /// stays in place, as Files offers) and pass it to the web app.
    func openDocument(at url: URL) {
        let scoped = url.startAccessingSecurityScopedResource()
        defer { if scoped { url.stopAccessingSecurityScopedResource() } }
        var readError: NSError?
        var data: Data?
        NSFileCoordinator().coordinate(readingItemAt: url, options: [], error: &readError) { readURL in
            data = try? Data(contentsOf: readURL)
        }
        guard let data = data, !data.isEmpty, data.count <= ViewController.maxDocumentBytes else {
            showOpenError(url.lastPathComponent)
            return
        }
        let document = (name: url.lastPathComponent, base64: data.base64EncodedString())
        if pageLoaded {
            deliver(document)
        } else {
            pendingDocuments.append(document)
        }
    }

    /// Hand a document to the web app (`wordpadxOpenFile`), retrying until the app has started:
    /// the page starts it asynchronously after it has loaded.
    private func deliver(_ document: (name: String, base64: String)) {
        let name = (try? JSONSerialization.data(withJSONObject: [document.name], options: []))
            .flatMap { String(data: $0, encoding: .utf8) }
            .map { String($0.dropFirst().dropLast()) } ?? "\"document.docx\""
        let js = "(function o(t){if(window.wordpadxOpenFile){wordpadxOpenFile(\(name),'\(document.base64)')}else if(t<300){setTimeout(function(){o(t+1)},200)}})(0)"
        webView.evaluateJavaScript(js, completionHandler: nil)
    }

    private func showOpenError(_ name: String) {
        let alert = UIAlertController(title: "Couldn't open \(name)", message: "The file couldn't be read, or it is larger than 200 MB.", preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "OK", style: .default))
        present(alert, animated: true)
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override var prefersHomeIndicatorAutoHidden: Bool {
        return false
    }
}
