import AppKit
import Foundation

func emit(_ value: String) {
  let data = Data((value + "\n").utf8)
  FileHandle.standardOutput.write(data)
}

let pasteboard = NSPasteboard.general
let payloadItems = (pasteboard.pasteboardItems ?? []).map { item in
  item.types.reduce(into: [String: String]()) { result, type in
    if let data = item.data(forType: type) {
      result[type.rawValue] = data.base64EncodedString()
    }
  }
}

let payload: [String: Any] = ["items": payloadItems]
let jsonData = try JSONSerialization.data(withJSONObject: payload, options: [])
emit(jsonData.base64EncodedString())
