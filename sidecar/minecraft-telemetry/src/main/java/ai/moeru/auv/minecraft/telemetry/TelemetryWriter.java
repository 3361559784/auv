package ai.moeru.auv.minecraft.telemetry;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;

public final class TelemetryWriter {
  private final Path outputPath;

  public TelemetryWriter(Path outputPath) {
    this.outputPath = outputPath;
  }

  public void append(TelemetrySample sample) throws IOException {
    Path parent = outputPath.getParent();
    if (parent != null) {
      Files.createDirectories(parent);
    }
    Files.writeString(
      outputPath,
      sample.toJsonLine() + System.lineSeparator(),
      StandardCharsets.UTF_8,
      StandardOpenOption.CREATE,
      StandardOpenOption.APPEND
    );
  }
}
