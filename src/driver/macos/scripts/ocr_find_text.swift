import Foundation
import Vision
import ImageIO

let imagePath = __IMAGE_PATH__
let rawQuery = __QUERY__
let exact = __EXACT__
let caseSensitive = __CASE_SENSITIVE__
let maxObservations = __MAX_OBSERVATIONS__
let cropEnabled = __CROP_ENABLED__
let cropX = __CROP_X__
let cropY = __CROP_Y__
let cropWidth = __CROP_WIDTH__
let cropHeight = __CROP_HEIGHT__

func sanitize(_ raw: String) -> String {
  raw
    .replacingOccurrences(of: "\t", with: " ")
    .replacingOccurrences(of: "\n", with: " ")
    .replacingOccurrences(of: "\r", with: " ")
    .trimmingCharacters(in: .whitespacesAndNewlines)
}

func foldConfusableScalar(_ scalar: UnicodeScalar) -> UnicodeScalar {
  switch scalar {
  case "|", "!", "l", "I":
    return "i"
  default:
    return scalar
  }
}

func normalizeForAnchorMatch(_ raw: String) -> String {
  let sanitized = sanitize(raw)
  let folded = String(String.UnicodeScalarView(sanitized.unicodeScalars.map(foldConfusableScalar)))
  let lowercased = caseSensitive ? folded : folded.lowercased()
  return String(
    lowercased.unicodeScalars.filter { scalar in
      CharacterSet.alphanumerics.contains(scalar)
    }
  )
}

let imageURL = URL(fileURLWithPath: imagePath)
guard
  let imageSource = CGImageSourceCreateWithURL(imageURL as CFURL, nil),
  let image = CGImageSourceCreateImageAtIndex(imageSource, 0, nil)
else {
  fputs("could not load image for OCR at \(imagePath)\n", stderr)
  exit(1)
}

func upscale(_ image: CGImage, factor: CGFloat) -> CGImage? {
  let width = Int((CGFloat(image.width) * factor).rounded())
  let height = Int((CGFloat(image.height) * factor).rounded())
  guard
    let colorSpace = image.colorSpace ?? CGColorSpace(name: CGColorSpace.sRGB),
    let context = CGContext(
      data: nil,
      width: width,
      height: height,
      bitsPerComponent: image.bitsPerComponent,
      bytesPerRow: 0,
      space: colorSpace,
      bitmapInfo: image.bitmapInfo.rawValue
    )
  else {
    return nil
  }

  context.interpolationQuality = .high
  context.draw(image, in: CGRect(x: 0, y: 0, width: width, height: height))
  return context.makeImage()
}

var workingImage = image
var cropOffsetX = 0
var cropOffsetY = 0
var ocrScaleFactor: CGFloat = 1.0

if cropEnabled {
  let cropRect = CGRect(x: cropX, y: cropY, width: cropWidth, height: cropHeight).integral
  guard cropRect.width > 0, cropRect.height > 0 else {
    fputs("invalid OCR crop rect \(cropRect)\n", stderr)
    exit(1)
  }
  guard let croppedImage = image.cropping(to: cropRect) else {
    fputs("could not crop OCR image to \(cropRect)\n", stderr)
    exit(1)
  }
  cropOffsetX = Int(cropRect.origin.x.rounded())
  cropOffsetY = Int(cropRect.origin.y.rounded())
  if let upscaledImage = upscale(croppedImage, factor: 2.0) {
    workingImage = upscaledImage
    ocrScaleFactor = 2.0
  } else {
    workingImage = croppedImage
  }
}

let normalizedQuery = caseSensitive ? rawQuery : rawQuery.lowercased()
let normalizedAnchorQuery = normalizeForAnchorMatch(rawQuery)

func matches(_ text: String) -> Bool {
  let normalizedText = caseSensitive ? text : text.lowercased()
  let normalizedAnchorText = normalizeForAnchorMatch(text)
  if exact {
    return normalizedText == normalizedQuery || normalizedAnchorText == normalizedAnchorQuery
  }
  return normalizedText.contains(normalizedQuery)
    || normalizedAnchorText.contains(normalizedAnchorQuery)
}

let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = true
request.recognitionLanguages = ["zh-Hans", "zh-Hant", "en-US"]
request.customWords = [rawQuery]
if #available(macOS 26.0, *) {
  request.automaticallyDetectsLanguage = true
}

let handler = VNImageRequestHandler(cgImage: workingImage, options: [:])
do {
  try handler.perform([request])
} catch {
  fputs("vision OCR failed: \(error)\n", stderr)
  exit(1)
}

let observations = (request.results as? [VNRecognizedTextObservation]) ?? []

print("recognizedAt=\(ISO8601DateFormatter().string(from: Date()))")
print("imagePath=\(imagePath)")
print("imageWidth=\(image.width)")
print("imageHeight=\(image.height)")
print("query=\(sanitize(rawQuery))")
print("exact=\(exact ? "true" : "false")")
print("caseSensitive=\(caseSensitive ? "true" : "false")")
print("normalizedQuery=\(normalizedAnchorQuery)")
if cropEnabled {
  print("cropRect=\(cropOffsetX),\(cropOffsetY),\(cropWidth),\(cropHeight)")
  print("ocrScaleFactor=\(String(format: "%.3f", ocrScaleFactor))")
}

var matchCount = 0
for observation in observations.prefix(maxObservations) {
  let candidates = observation.topCandidates(5)
  guard
    let candidate = candidates.first(where: { candidate in
      let text = sanitize(candidate.string)
      return !text.isEmpty && matches(text)
    })
  else { continue }
  let text = sanitize(candidate.string)

  let boundingBox = observation.boundingBox
  let workingX = Int((boundingBox.minX * CGFloat(workingImage.width)).rounded())
  let workingY = Int(((1.0 - boundingBox.maxY) * CGFloat(workingImage.height)).rounded())
  let workingWidth = Int((boundingBox.width * CGFloat(workingImage.width)).rounded())
  let workingHeight = Int((boundingBox.height * CGFloat(workingImage.height)).rounded())

  let x = cropOffsetX + Int((CGFloat(workingX) / ocrScaleFactor).rounded())
  let y = cropOffsetY + Int((CGFloat(workingY) / ocrScaleFactor).rounded())
  let width = Int((CGFloat(workingWidth) / ocrScaleFactor).rounded())
  let height = Int((CGFloat(workingHeight) / ocrScaleFactor).rounded())

  print(
    "match\t\(matchCount)\t\(text)\t\(String(format: "%.6f", candidate.confidence))\t\(x)\t\(y)\t\(width)\t\(height)"
  )
  matchCount += 1
}

print("matchCount=\(matchCount)")
