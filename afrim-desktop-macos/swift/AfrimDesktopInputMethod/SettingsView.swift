import SwiftUI

/// Settings view for the AfrimDesktop input method.
/// Accessible from the input method menu bar icon.
struct SettingsView: View {
    var body: some View {
        Form {
            Section("Afrim dataset") {
                Text("Set `config_path` in the [afrim] section of ~/Library/Application Support/org.fodydev.afrim-desktop/config.toml, or place your dataset at afrim/config.toml in that folder.")
                Text("Datasets: https://github.com/fodydev/afrim-data")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Section("Keys") {
                Text("Space / Return: commit · Tab, ↑ ↓: choose a suggestion · Esc: cancel · Ctrl+Shift+Space: toggle afrim / direct input")
                    .font(.caption)
            }
        }
        .formStyle(.grouped)
        .frame(width: 400, height: 300)
        .padding()
    }
}
