import SwiftUI

public struct ToastView: View {
    let icon: String
    let iconColor: Color
    let message: String

    public init(icon: String, iconColor: Color, message: String) {
        self.icon = icon
        self.iconColor = iconColor
        self.message = message
    }

    public var body: some View {
        VStack(spacing: 12) {
            Image(systemName: icon)
                .font(.system(size: 48))
                .foregroundStyle(iconColor)

            Text(message)
                .font(.headline)
        }
        .padding(32)
        .frame(maxWidth: 280)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 16))
        .transition(.scale.combined(with: .opacity))
    }
}

#Preview("Success") {
    ZStack {
        Color.gray.opacity(0.2)
        ToastView(
            icon: "checkmark.circle.fill",
            iconColor: .green,
            message: "Transaction Sent!"
        )
    }
}

#Preview("Success Dark") {
    ZStack {
        Color.gray.opacity(0.2)
        ToastView(
            icon: "checkmark.circle.fill",
            iconColor: .green,
            message: "Transaction Sent!"
        )
    }
    .preferredColorScheme(.dark)
}

#Preview("Error") {
    ZStack {
        Color.gray.opacity(0.2)
        ToastView(
            icon: "xmark.circle.fill",
            iconColor: .red,
            message: "Error Occurred"
        )
    }
}
