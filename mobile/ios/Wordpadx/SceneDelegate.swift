import UIKit

class SceneDelegate: UIResponder, UIWindowSceneDelegate {
    var window: UIWindow?

    func scene(
        _ scene: UIScene,
        willConnectTo session: UISceneSession,
        options connectionOptions: UIScene.ConnectionOptions
    ) {
        guard let windowScene = (scene as? UIWindowScene) else { return }

        let window = UIWindow(windowScene: windowScene)
        let controller = ViewController()
        window.rootViewController = controller
        self.window = window
        window.makeKeyAndVisible()
        // Launched by opening a document from Files or another app ("Open in Wordpadx").
        for context in connectionOptions.urlContexts {
            controller.openDocument(at: context.url)
        }
    }

    /// A document opened from Files or another app while Wordpadx is running.
    func scene(_ scene: UIScene, openURLContexts URLContexts: Set<UIOpenURLContext>) {
        guard let controller = window?.rootViewController as? ViewController else { return }
        for context in URLContexts {
            controller.openDocument(at: context.url)
        }
    }

    func sceneDidDisconnect(_ scene: UIScene) {}
    func sceneDidBecomeActive(_ scene: UIScene) {}
    func sceneWillResignActive(_ scene: UIScene) {}
    func sceneWillEnterForeground(_ scene: UIScene) {}
    func sceneDidEnterBackground(_ scene: UIScene) {}
}
