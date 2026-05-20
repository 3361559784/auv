import AppKit
import ApplicationServices
import Foundation

let pid = pid_t(__PID__)
let pathRaw = __PATH__
let expectedRole = __EXPECTED_ROLE__
let actionName = __ACTION_NAME__

func sanitize(_ raw: String?) -> String {
  guard let raw else { return "" }
  return raw
    .replacingOccurrences(of: "\t", with: " ")
    .replacingOccurrences(of: "\n", with: " ")
    .replacingOccurrences(of: "\r", with: " ")
    .trimmingCharacters(in: .whitespacesAndNewlines)
}

func attributeValue(_ element: AXUIElement, _ attribute: String) -> CFTypeRef? {
  var value: CFTypeRef?
  let result = AXUIElementCopyAttributeValue(element, attribute as CFString, &value)
  guard result == .success else { return nil }
  return value
}

func axElementAttribute(_ element: AXUIElement, _ attribute: String) -> AXUIElement? {
  guard let rawValue = attributeValue(element, attribute) else { return nil }
  guard CFGetTypeID(rawValue) == AXUIElementGetTypeID() else { return nil }
  return unsafeBitCast(rawValue, to: AXUIElement.self)
}

func elementArrayAttribute(_ element: AXUIElement, _ attribute: String) -> [AXUIElement] {
  guard let rawValue = attributeValue(element, attribute) else { return [] }
  guard let array = rawValue as? NSArray else { return [] }
  return array.compactMap { item in
    let value = item as CFTypeRef
    guard CFGetTypeID(value) == AXUIElementGetTypeID() else { return nil }
    return unsafeBitCast(value, to: AXUIElement.self)
  }
}

func stringAttribute(_ element: AXUIElement, _ attribute: String) -> String {
  if let stringValue = attributeValue(element, attribute) as? String {
    return sanitize(stringValue)
  }
  return ""
}

func firstWindow(_ appElement: AXUIElement) -> AXUIElement? {
  if let focused = axElementAttribute(appElement, kAXFocusedWindowAttribute as String) {
    return focused
  }
  return elementArrayAttribute(appElement, kAXWindowsAttribute as String).first
}

func fail(_ message: String, code: Int32 = 1) -> Never {
  fputs("\(message)\n", stderr)
  exit(code)
}

// Walk the path segments from the root.
//
// Path format matches observe_window_tree.swift: "0.1.2.3" — first segment
// is the root window (or app element when no window exists), each subsequent
// segment is a child index.
let segments = pathRaw.split(separator: ".").map { String($0) }
guard let firstSegment = segments.first, firstSegment == "0" else {
  fail("ax_press_path expects a path beginning with \"0\"; got \"\(pathRaw)\"")
}

let appElement = AXUIElementCreateApplication(pid)
let rootElement = firstWindow(appElement) ?? appElement

var current = rootElement
for (offset, segment) in segments.dropFirst().enumerated() {
  guard let index = Int(segment) else {
    fail("ax_press_path encountered non-integer segment \"\(segment)\" at offset \(offset) in path \"\(pathRaw)\"")
  }
  let kids = elementArrayAttribute(current, kAXChildrenAttribute as String)
  if index < 0 || index >= kids.count {
    fail("ax_press_path index \(index) out of range at offset \(offset); element has \(kids.count) child(ren) — AX tree likely shifted since observation")
  }
  current = kids[index]
}

let actualRole = stringAttribute(current, kAXRoleAttribute as String)
let actualSubrole = stringAttribute(current, kAXSubroleAttribute as String)
let actualTitle = stringAttribute(current, kAXTitleAttribute as String)
let actualDescription = stringAttribute(current, kAXDescriptionAttribute as String)
let actualIdentifier = stringAttribute(current, kAXIdentifierAttribute as String)

if !expectedRole.isEmpty && actualRole != expectedRole {
  fail("ax_press_path expected role \"\(expectedRole)\" at path \"\(pathRaw)\", got \"\(actualRole)\" — AX tree likely shifted since observation", code: 2)
}

var actionNames: CFArray?
let listResult = AXUIElementCopyActionNames(current, &actionNames)
let actions: [String]
if listResult == .success, let raw = actionNames as? [String] {
  actions = raw
} else {
  actions = []
}
print("availableActions=\(actions.joined(separator: ","))")

if !actions.contains(actionName) {
  fail("ax_press_path target does not support action \"\(actionName)\"; available=[\(actions.joined(separator: ","))]", code: 3)
}

let pressResult = AXUIElementPerformAction(current, actionName as CFString)
if pressResult != .success {
  fail("AXUIElementPerformAction(\"\(actionName)\") returned \(pressResult.rawValue)", code: 4)
}

print("pressResult=success")
print("performedAction=\(actionName)")
print("path=\(pathRaw)")
print("pid=\(pid)")
print("role=\(actualRole)")
print("subrole=\(actualSubrole)")
print("title=\(actualTitle)")
print("description=\(actualDescription)")
print("identifier=\(actualIdentifier)")
