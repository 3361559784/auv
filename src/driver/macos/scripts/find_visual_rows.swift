import CoreGraphics
import Foundation
import ImageIO

let imagePath = __IMAGE_PATH__
let cropEnabled = __CROP_ENABLED__
let cropX = __CROP_X__
let cropY = __CROP_Y__
let cropWidth = __CROP_WIDTH__
let cropHeight = __CROP_HEIGHT__

let imageURL = URL(fileURLWithPath: imagePath)
guard
  let imageSource = CGImageSourceCreateWithURL(imageURL as CFURL, nil),
  let image = CGImageSourceCreateImageAtIndex(imageSource, 0, nil)
else {
  fputs("could not load image for visual row detection at \(imagePath)\n", stderr)
  exit(1)
}

var workingImage = image
var cropOffsetX = 0
var cropOffsetY = 0

if cropEnabled {
  let cropRect = CGRect(x: cropX, y: cropY, width: cropWidth, height: cropHeight).integral
  guard cropRect.width > 0, cropRect.height > 0 else {
    fputs("invalid visual-row crop rect \(cropRect)\n", stderr)
    exit(1)
  }
  guard let croppedImage = image.cropping(to: cropRect) else {
    fputs("could not crop visual-row image to \(cropRect)\n", stderr)
    exit(1)
  }
  workingImage = croppedImage
  cropOffsetX = Int(cropRect.origin.x.rounded())
  cropOffsetY = Int(cropRect.origin.y.rounded())
}

guard
  let colorSpace = CGColorSpace(name: CGColorSpace.sRGB),
  let context = CGContext(
    data: nil,
    width: workingImage.width,
    height: workingImage.height,
    bitsPerComponent: 8,
    bytesPerRow: 0,
    space: colorSpace,
    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
  )
else {
  fputs("could not allocate bitmap context for visual row detection\n", stderr)
  exit(1)
}

context.draw(workingImage, in: CGRect(x: 0, y: 0, width: workingImage.width, height: workingImage.height))

guard let rawData = context.data else {
  fputs("visual row bitmap context did not expose pixel data\n", stderr)
  exit(1)
}

let width = workingImage.width
let height = workingImage.height
let bytesPerRow = context.bytesPerRow
let pixels = rawData.bindMemory(to: UInt8.self, capacity: bytesPerRow * height)

func pixelComponents(x: Int, y: Int) -> (r: Double, g: Double, b: Double, a: Double) {
  let offset = y * bytesPerRow + x * 4
  return (
    Double(pixels[offset]) / 255.0,
    Double(pixels[offset + 1]) / 255.0,
    Double(pixels[offset + 2]) / 255.0,
    Double(pixels[offset + 3]) / 255.0
  )
}

func luminance(_ pixel: (r: Double, g: Double, b: Double, a: Double)) -> Double {
  0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b
}

func saturation(_ pixel: (r: Double, g: Double, b: Double, a: Double)) -> Double {
  let maxValue = max(pixel.r, max(pixel.g, pixel.b))
  let minValue = min(pixel.r, min(pixel.g, pixel.b))
  return maxValue - minValue
}

func edgeStrength(x: Int, y: Int) -> Double {
  let pixel = pixelComponents(x: x, y: y)
  let rightPixel = pixelComponents(x: min(width - 1, x + 1), y: y)
  let downPixel = pixelComponents(x: x, y: min(height - 1, y + 1))
  return
    abs(luminance(pixel) - luminance(rightPixel))
    + abs(luminance(pixel) - luminance(downPixel))
}

func isActive(x: Int, y: Int) -> Bool {
  let pixel = pixelComponents(x: x, y: y)
  guard pixel.a > 0.05 else { return false }
  return edgeStrength(x: x, y: y) >= 0.10 || saturation(pixel) > 0.24
}

let stripLeft = max(0, Int((Double(width) * 0.02).rounded()))
let stripRight = min(width, max(stripLeft + 1, Int((Double(width) * 0.24).rounded())))
let stripWidth = max(1, stripRight - stripLeft)

var rowSignal = Array(repeating: 0.0, count: height)
for y in 0..<height {
  var activeCount = 0
  for x in stripLeft..<stripRight {
    if isActive(x: x, y: y) {
      activeCount += 1
    }
  }
  rowSignal[y] = Double(activeCount) / Double(stripWidth)
}

let smoothingRadius = 2
var smoothedSignal = Array(repeating: 0.0, count: height)
for y in 0..<height {
  let lower = max(0, y - smoothingRadius)
  let upper = min(height - 1, y + smoothingRadius)
  let window = rowSignal[lower...upper]
  let total = window.reduce(0.0, +)
  smoothedSignal[y] = total / Double(window.count)
}

let rowThreshold = 0.018
let maxGap = 10
let minBandHeight = 28
let maxBandHeight = 220

var rawBands = [(start: Int, end: Int)]()
var bandStart: Int? = nil
var gap = 0
for y in 0..<height {
  if smoothedSignal[y] >= rowThreshold {
    if bandStart == nil {
      bandStart = y
    }
    gap = 0
  } else if let start = bandStart {
    gap += 1
    if gap > maxGap {
      rawBands.append((start: start, end: y - gap))
      bandStart = nil
      gap = 0
    }
  }
}
if let start = bandStart {
  rawBands.append((start: start, end: height - 1))
}

let filteredBands = rawBands.filter { band in
  let bandHeight = band.end - band.start + 1
  return bandHeight >= minBandHeight && bandHeight <= maxBandHeight
}

print("detectedAt=\(ISO8601DateFormatter().string(from: Date()))")
print("imagePath=\(imagePath)")
print("imageWidth=\(image.width)")
print("imageHeight=\(image.height)")
print("rowStrategy=visual-bands")
if cropEnabled {
  print("cropRect=\(cropOffsetX),\(cropOffsetY),\(cropWidth),\(cropHeight)")
}
print("analysisStrip=\(stripLeft),0,\(stripWidth),\(height)")

var emittedRowCount = 0
for (bandIndex, band) in filteredBands.enumerated() {
  let bandTop = max(0, band.start - 6)
  let bandBottom = min(height - 1, band.end + 6)
  let bandHeight = bandBottom - bandTop + 1

  var leftX: Int?
  var rightX: Int?
  for x in 0..<width {
    var activeCount = 0
    for y in bandTop...bandBottom {
      if isActive(x: x, y: y) {
        activeCount += 1
      }
    }
    let columnDensity = Double(activeCount) / Double(max(1, bandHeight))
    if columnDensity >= 0.04 {
      if leftX == nil {
        leftX = x
      }
      rightX = x
    }
  }

  let visualLeft = max(0, (leftX ?? stripLeft) - 8)
  let visualRight = min(width - 1, (rightX ?? (width - 1)) + 8)
  let visualWidth = max(1, visualRight - visualLeft + 1)
  let peakDensity = smoothedSignal[band.start...band.end].max() ?? 0.0

  print(
    "row\t\(bandIndex)\t\(cropOffsetX + visualLeft)\t\(cropOffsetY + bandTop)\t\(visualWidth)\t\(bandHeight)\t\(String(format: "%.6f", peakDensity))"
  )
  emittedRowCount += 1
}

print("rowCount=\(emittedRowCount)")
