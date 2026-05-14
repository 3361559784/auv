import AppKit
import Foundation

func fail(_ message: String) -> Never {
  FileHandle.standardError.write(Data((message + "\n").utf8))
  exit(1)
}

let payload = __PAYLOAD__

guard let payloadData = Data(base64Encoded: payload) else {
  fail("invalid clipboard payload base64")
}

guard let decoded = try JSONSerialization.jsonObject(with: payloadData) as? [String: Any] else {
  fail("invalid clipboard payload json")
}

guard let payloadItems = decoded["items"] as? [[String: String]] else {
  fail("clipboard payload is missing items")
}

let pasteboard = NSPasteboard.general
pasteboard.clearContents()

var items = [NSPasteboardItem]()
for payloadItem in payloadItems {
  let item = NSPasteboardItem()
  for (typeRawValue, encodedData) in payloadItem {
    guard let data = Data(base64Encoded: encodedData) else {
      fail("invalid base64 clipboard item for type \(typeRawValue)")
    }
    item.setData(data, forType: NSPasteboard.PasteboardType(typeRawValue))
  }
  items.append(item)
}

if !items.isEmpty {
  let success = pasteboard.writeObjects(items)
  if !success {
    fail("failed to restore clipboard items")
  }
}

FileHandle.standardOutput.write(Data("restored\n".utf8))
