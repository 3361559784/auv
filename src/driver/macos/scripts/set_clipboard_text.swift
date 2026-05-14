import AppKit
import Foundation

let text = __TEXT__
let pasteboard = NSPasteboard.general
pasteboard.clearContents()
pasteboard.setString(text, forType: .string)
FileHandle.standardOutput.write(Data("ok\n".utf8))
