import CoreGraphics
import Foundation

struct Options {
  var targetX: Double?
  var targetY: Double?
  var deltaX: Double = 420
  var deltaY: Double = 0
  var repeats: Int = 5
  var intervalMs: UInt32 = 450
  var settleMs: UInt32 = 0
}

func usage() -> Never {
  fputs(
    """
    usage: swift scripts/local/cursor-warp-jitter-smoke.swift [options]

    Options:
      --target-x <number>    Absolute global logical target x.
      --target-y <number>    Absolute global logical target y.
      --delta-x <number>     Target x offset from current cursor when no absolute target is supplied. Default: 420.
      --delta-y <number>     Target y offset from current cursor when no absolute target is supplied. Default: 0.
      --repeat <count>       Number of warp-and-restore samples. Default: 5.
      --interval-ms <ms>     Delay between samples. Default: 450.
      --settle-ms <ms>       Optional delay between target warp and restore. Default: 0.

    This smoke intentionally does not click, use overlay, or require Accessibility.
    It only tests whether CGWarpMouseCursorPosition(target) followed by immediate restore causes visible flicker.

    """,
    stderr
  )
  exit(64)
}

func readNumber(_ args: [String], _ index: inout Int, _ flag: String) -> Double {
  guard index + 1 < args.count, let value = Double(args[index + 1]) else {
    usage()
  }
  index += 2
  return value
}

func readUInt(_ args: [String], _ index: inout Int, _ flag: String) -> UInt32 {
  let value = readNumber(args, &index, flag)
  guard value >= 0 else {
    usage()
  }
  return UInt32(value.rounded())
}

func readInt(_ args: [String], _ index: inout Int, _ flag: String) -> Int {
  let value = readNumber(args, &index, flag)
  guard value >= 1 else {
    usage()
  }
  return Int(value.rounded())
}

func parseOptions() -> Options {
  var options = Options()
  let args = Array(CommandLine.arguments.dropFirst())
  var index = 0

  while index < args.count {
    switch args[index] {
    case "--target-x":
      options.targetX = readNumber(args, &index, args[index])
    case "--target-y":
      options.targetY = readNumber(args, &index, args[index])
    case "--delta-x":
      options.deltaX = readNumber(args, &index, args[index])
    case "--delta-y":
      options.deltaY = readNumber(args, &index, args[index])
    case "--repeat":
      options.repeats = readInt(args, &index, args[index])
    case "--interval-ms":
      options.intervalMs = readUInt(args, &index, args[index])
    case "--settle-ms":
      options.settleMs = readUInt(args, &index, args[index])
    case "--help", "-h":
      usage()
    default:
      usage()
    }
  }

  if (options.targetX == nil) != (options.targetY == nil) {
    usage()
  }

  return options
}

func sleepMs(_ milliseconds: UInt32) {
  if milliseconds > 0 {
    usleep(milliseconds * 1000)
  }
}

let options = parseOptions()
let original = CGEvent(source: nil)?.location ?? CGPoint(x: 0, y: 0)
let target = CGPoint(
  x: options.targetX ?? (original.x + options.deltaX),
  y: options.targetY ?? (original.y + options.deltaY)
)

print("cursorWarpJitterSmoke=true")
print("original=\(String(format: "%.3f", original.x)),\(String(format: "%.3f", original.y))")
print("target=\(String(format: "%.3f", target.x)),\(String(format: "%.3f", target.y))")
print("repeats=\(options.repeats)")
print("intervalMs=\(options.intervalMs)")
print("settleMs=\(options.settleMs)")
print("action=CGWarpMouseCursorPosition(target)->CGWarpMouseCursorPosition(original)")
print("manualObservation=watch whether the real cursor visibly flashes at target before returning")
fflush(stdout)

for sample in 1...options.repeats {
  let started = DispatchTime.now().uptimeNanoseconds
  CGWarpMouseCursorPosition(target)
  sleepMs(options.settleMs)
  CGWarpMouseCursorPosition(original)
  let ended = DispatchTime.now().uptimeNanoseconds
  let elapsedMs = Double(ended - started) / 1_000_000.0
  print("sample\t\(sample)\telapsedMs=\(String(format: "%.3f", elapsedMs))")
  fflush(stdout)
  sleepMs(options.intervalMs)
}

let final = CGEvent(source: nil)?.location ?? original
print("final=\(String(format: "%.3f", final.x)),\(String(format: "%.3f", final.y))")
print("restored=\(abs(final.x - original.x) < 1.0 && abs(final.y - original.y) < 1.0)")
