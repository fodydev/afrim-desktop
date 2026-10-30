import Cocoa
import InputMethodKit

// IMKServer manages the connection between the input method and client apps
let bundleId = Bundle.main.bundleIdentifier ?? "org.fodydev.inputmethod.AfrimDesktop"
let connectionName = "AfrimDesktop_Connection"

guard let server = IMKServer(name: connectionName, bundleIdentifier: bundleId) else {
    NSLog("Afrim Desktop: Failed to create IMKServer")
    exit(1)
}

NSLog("Afrim Desktop: IMKServer started (connection: \(connectionName))")

// Keep a reference to prevent deallocation
_ = server

NSApplication.shared.run()
