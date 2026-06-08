import Foundation
import LocalAuthentication

private let humanApprovalMechanism = "local_auth_device_owner_authentication"

private func nativeHumanApprovalResponse(
  status: NativeHumanApprovalStatus,
  approvedAtUnixMs: Int64 = 0,
  errorMessage: String? = nil,
  recoveryHint: String? = nil
) -> NativeHumanApprovalResponse {
  NativeHumanApprovalResponse(
    status: status,
    approved_at_unix_ms: approvedAtUnixMs,
    mechanism: humanApprovalMechanism.intoRustString(),
    error_message: errorMessage?.intoRustString(),
    recovery_hint: recoveryHint?.intoRustString()
  )
}

private func nativeHumanApprovalTimestampMs() -> Int64 {
  Int64((Date().timeIntervalSince1970 * 1000.0).rounded())
}

private func nativeHumanApprovalUnavailableRecoveryHint() -> String {
  "enable device-owner authentication for this macOS session and retry"
}

func request_human_approval(reason: RustString, timeout_ms: UInt64) -> NativeHumanApprovalResponse {
  let localizedReason = nativeSanitize(reason.toString())
  let prompt = localizedReason.isEmpty
    ? "Approve one AUV candidate action."
    : localizedReason
  let timeoutSeconds = Double(timeout_ms) / 1000.0
  let context = LAContext()
  var evaluationError: NSError?

  guard context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &evaluationError) else {
    return nativeHumanApprovalResponse(
      status: .Unavailable,
      errorMessage: nativeSanitize(evaluationError?.localizedDescription),
      recoveryHint: nativeHumanApprovalUnavailableRecoveryHint()
    )
  }

  let semaphore = DispatchSemaphore(value: 0)
  var evaluationSucceeded = false
  var callbackError: Error?

  context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: prompt) { success, error in
    evaluationSucceeded = success
    callbackError = error
    semaphore.signal()
  }

  if semaphore.wait(timeout: .now() + timeoutSeconds) == .timedOut {
    context.invalidate()
    return nativeHumanApprovalResponse(
      status: .TimedOut,
      errorMessage: "timed out waiting for human approval",
      recoveryHint: "invoke the command again and approve before the timeout expires"
    )
  }

  if evaluationSucceeded {
    return nativeHumanApprovalResponse(
      status: .Approved,
      approvedAtUnixMs: nativeHumanApprovalTimestampMs()
    )
  }

  if let laError = callbackError as? LAError {
    switch laError.code {
    case .userCancel, .userFallback, .appCancel, .systemCancel, .authenticationFailed:
      return nativeHumanApprovalResponse(
        status: .Declined,
        errorMessage: nativeSanitize(laError.localizedDescription),
        recoveryHint: "invoke the command again and approve the system prompt to continue"
      )
    default:
      return nativeHumanApprovalResponse(
        status: .Unavailable,
        errorMessage: nativeSanitize(laError.localizedDescription),
        recoveryHint: nativeHumanApprovalUnavailableRecoveryHint()
      )
    }
  }

  if let callbackError {
    return nativeHumanApprovalResponse(
      status: .Unavailable,
      errorMessage: nativeSanitize(callbackError.localizedDescription),
      recoveryHint: nativeHumanApprovalUnavailableRecoveryHint()
    )
  }

  return nativeHumanApprovalResponse(
    status: .Declined,
    errorMessage: "human approval was not granted",
    recoveryHint: "invoke the command again and approve the system prompt to continue"
  )
}
