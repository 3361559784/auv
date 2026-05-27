import ApplicationServices
import CoreGraphics
import Foundation
import ScreenCaptureKit

func probe_permissions() -> NativePermissionProbeResponse {
  NativePermissionProbeResponse(
    screen_recording: CGPreflightScreenCaptureAccess()
      ? NativePermissionStatus.Granted
      : NativePermissionStatus.Missing,
    screen_capture_kit: probeScreenCaptureKitAccess()
      ? NativePermissionStatus.Granted
      : NativePermissionStatus.Missing,
    accessibility: AXIsProcessTrusted()
      ? NativePermissionStatus.Granted
      : NativePermissionStatus.Missing
  )
}

private func probeScreenCaptureKitAccess() -> Bool {
  guard #available(macOS 12.3, *) else {
    return false
  }

  let semaphore = DispatchSemaphore(value: 0)
  var granted = false

  SCShareableContent.getWithCompletionHandler { content, error in
    granted = error == nil && content != nil
    semaphore.signal()
  }

  if semaphore.wait(timeout: .now() + .seconds(3)) == .timedOut {
    return false
  }
  return granted
}
