import SwiftUI

public struct AddressView: View {
    let address: String
    @State private var showingCopiedFeedback = false
    
    public init(address: String) {
        self.address = address
    }
    
    public var body: some View {
        Text(address)
            .lineLimit(1)
            .truncationMode(.middle)
            .padding()
            .background {
                RoundedRectangle(cornerRadius: 12)
                    .fill(.secondary.opacity(0.1))
            }
            .onTapGesture {
                UIPasteboard.general.string = address
                withAnimation {
                    showingCopiedFeedback = true
                }

                Task {
                    try? await Task.sleep(for: .seconds(2))
                    withAnimation {
                        showingCopiedFeedback = false
                    }
                }
            }
            .overlay {
                if showingCopiedFeedback {
                    Text("Copied!")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(.thinMaterial)
                        .cornerRadius(4)
                        .transition(.move(edge: .top).combined(with: .opacity))
                }
            }
    }
}

#Preview {
    AddressView(address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .padding()
} 
