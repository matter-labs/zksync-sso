import SwiftUI

struct PasskeyPresentationContext: ViewModifier {
    let passkeyManager: PasskeyManager

    init(passkeyManager: PasskeyManager) {
        self.passkeyManager = passkeyManager
        print("PasskeyPresentationContext initialized")
    }

    func body(content: Content) -> some View {
        print("PasskeyPresentationContext body called")
        return
            content
            .onAppear {
                print("PasskeyPresentationContext onAppear")
                updateWindow()
            }
            .onChange(of: UIApplication.shared.connectedScenes) { _ in
                print("Scenes changed, updating window")
                updateWindow()
            }
            .onDisappear {
                print("PasskeyPresentationContext onDisappear")
            }
    }

    private func updateWindow() {
        #if os(iOS)
            print("Getting window...")
            let window = UIApplication.shared.connectedScenes
                .compactMap { $0 as? UIWindowScene }
                .first(where: { $0.activationState == .foregroundActive })?
                .windows
                .first(where: \.isKeyWindow)

            print("Found window: \(String(describing: window))")
            passkeyManager.setPresentationAnchor(window)
        #endif
    }
}

extension View {
    func passkeyPresentation(_ manager: PasskeyManager) -> some View {
        modifier(PasskeyPresentationContext(passkeyManager: manager))
    }
}
