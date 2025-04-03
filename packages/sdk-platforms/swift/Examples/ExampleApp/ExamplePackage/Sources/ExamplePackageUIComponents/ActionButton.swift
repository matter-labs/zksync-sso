import SwiftUI

public struct ActionButton: View {
    let title: String
    let progressTitle: String?
    let icon: String?
    let isLoading: Bool
    let isDisabled: Bool
    let style: Style
    let action: () -> Void

    public enum Style: Sendable {
        case prominent
        case destructive
        case plain

        @MainActor @ViewBuilder
        func applyStyle<V: View>(to view: V) -> some View {
            switch self {
            case .prominent:
                view.buttonStyle(.borderedProminent)
            case .destructive:
                view.buttonStyle(.borderless)
                    .foregroundStyle(.red)
            case .plain:
                view.buttonStyle(.bordered)
            }
        }
    }

    public init(
        title: String,
        progressTitle: String? = nil,
        icon: String? = nil,
        isLoading: Bool = false,
        isDisabled: Bool = false,
        style: Style = .prominent,
        action: @escaping () -> Void
    ) {
        self.title = title
        self.progressTitle = progressTitle
        self.icon = icon
        self.isLoading = isLoading
        self.isDisabled = isDisabled
        self.style = style
        self.action = action
    }

    public var body: some View {
        Section {
            style.applyStyle(
                to:
                    Button(action: action) {
                        HStack(spacing: 8) {
                            Spacer()
                            if isLoading {
                                ProgressView()
                                    .padding(.trailing, 4)
                            }

                            if let icon = icon {
                                Image(systemName: icon)
                            }

                            Text(isLoading ? (progressTitle ?? title) : title)
                                .font(.headline)
                            Spacer()
                        }
                        .frame(maxWidth: .infinity)
                        .frame(height: 44)
                    }
            )
            .listRowInsets(EdgeInsets())
            .listRowBackground(Color.clear)
            .disabled(isDisabled || isLoading)
        }
    }
}

#Preview("Styles") {
    Form {
        Group {
            ActionButton(
                title: "Prominent Action",
                progressTitle: "Processing...",
                icon: "arrow.right.circle.fill",
                style: .prominent,
                action: {}
            )

            ActionButton(
                title: "Destructive Action",
                progressTitle: "Deleting...",
                icon: "trash.fill",
                style: .destructive,
                action: {}
            )

            ActionButton(
                title: "Plain Action",
                progressTitle: "Working...",
                icon: "gear",
                style: .plain,
                action: {}
            )
        }
    }
}

#Preview("States") {
    Form {
        Group {
            ActionButton(
                title: "Normal State",
                icon: "checkmark.circle.fill",
                action: {}
            )

            ActionButton(
                title: "Loading State",
                progressTitle: "Processing...",
                icon: "arrow.clockwise",
                isLoading: true,
                action: {}
            )

            ActionButton(
                title: "Disabled State",
                icon: "xmark.circle.fill",
                isDisabled: true,
                action: {}
            )
        }
    }
}

#Preview("No Icon") {
    Form {
        ActionButton(
            title: "Action Without Icon",
            progressTitle: "Processing...",
            isLoading: false,
            action: {}
        )
    }
}
