import UIKit
import WebKit

class ViewController: UIViewController, WKNavigationDelegate, WKUIDelegate, UIDocumentPickerDelegate {
    private var webView: WKWebView!

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

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override var prefersHomeIndicatorAutoHidden: Bool {
        return false
    }
}
