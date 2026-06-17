package ai.moeru.auv.minecraft.telemetry;

import java.io.IOException;
import java.nio.FloatBuffer;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;

import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.rendering.v1.WorldRenderEvents;
import net.fabricmc.fabric.api.client.rendering.v1.WorldRenderContext;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.item.ItemStack;
import net.minecraft.registry.Registries;
import net.minecraft.util.hit.BlockHitResult;
import net.minecraft.util.hit.HitResult;
import net.minecraft.util.math.BlockPos;
import org.joml.Matrix4f;
import org.lwjgl.BufferUtils;

public final class TelemetryRecorder {
  private static final long RENDER_SAMPLE_INTERVAL_NS = 50_000_000L;
  private static final Object SAMPLE_LOCK = new Object();
  private static volatile boolean started = false;
  private static TelemetrySample latestTickSample;
  private static TelemetryWriter writer;
  private static long lastRenderSampleNs = Long.MIN_VALUE;

  private TelemetryRecorder() {}

  public static synchronized void start() {
    if (started) {
      return;
    }
    started = true;
    ClientTickEvents.END_CLIENT_TICK.register(TelemetryRecorder::recordTick);
    WorldRenderEvents.START.register(TelemetryRecorder::recordRender);
  }

  private static void recordTick(MinecraftClient client) {
    if (client.player == null || client.world == null || client.getWindow() == null) {
      return;
    }

    TelemetrySample sample = new TelemetrySample();
    sample.spatialFrameId = String.format("frame-%d-%d", client.world.getTime(), System.nanoTime());
    sample.worldTick = client.world.getTime();
    sample.monotonicTimestampMs = System.nanoTime() / 1_000_000L;
    sample.viewportWidth = client.getWindow().getFramebufferWidth();
    sample.viewportHeight = client.getWindow().getFramebufferHeight();
    populatePlayerPose(client.player, sample);
    populateRaycast(client, sample);
    populateInventory(client.player, sample);
    synchronized (SAMPLE_LOCK) {
      latestTickSample = sample;
    }
  }

  private static void recordRender(WorldRenderContext context) {
    MinecraftClient client = MinecraftClient.getInstance();
    if (client.getWindow() == null || client.player == null || client.world == null) {
      return;
    }

    long nowNs = System.nanoTime();
    synchronized (SAMPLE_LOCK) {
      if (nowNs - lastRenderSampleNs < RENDER_SAMPLE_INTERVAL_NS) {
        return;
      }
      lastRenderSampleNs = nowNs;
    }

    TelemetrySample sample;
    synchronized (SAMPLE_LOCK) {
      if (latestTickSample == null) {
        return;
      }
      sample = latestTickSample;
      latestTickSample = null;
    }

    sample.monotonicTimestampMs = nowNs / 1_000_000L;
    Matrix4f projectionMatrix = new Matrix4f(context.projectionMatrix());
    Matrix4f viewMatrix = new Matrix4f(context.positionMatrix());

    copyMatrix(viewMatrix, sample.viewMatrix);
    copyMatrix(projectionMatrix, sample.projectionMatrix);
    sample.viewportWidth = client.getWindow().getFramebufferWidth();
    sample.viewportHeight = client.getWindow().getFramebufferHeight();

    try {
      telemetryWriter(client).append(sample);
    } catch (IOException ignored) {
      // NOTICE(mc1-telemetry-gate): first gate is best-effort append-only sampling; hard failure/reporting can tighten after a real sample path exists.
    }
  }

  private static void populatePlayerPose(ClientPlayerEntity player, TelemetrySample sample) {
    sample.eyeX = player.getX();
    sample.eyeY = player.getEyeY();
    sample.eyeZ = player.getZ();
    sample.yaw = player.getYaw();
    sample.pitch = player.getPitch();
  }

  private static void populateRaycast(MinecraftClient client, TelemetrySample sample) {
    if (client.crosshairTarget == null || client.crosshairTarget.getType() != HitResult.Type.BLOCK) {
      return;
    }

    BlockHitResult hitResult = (BlockHitResult) client.crosshairTarget;
    BlockState blockState = client.world.getBlockState(hitResult.getBlockPos());
    sample.raycastBlockX = hitResult.getBlockPos().getX();
    sample.raycastBlockY = hitResult.getBlockPos().getY();
    sample.raycastBlockZ = hitResult.getBlockPos().getZ();
    sample.raycastFace = hitResult.getSide().asString();
    sample.raycastBlockId = Registries.BLOCK.getId(blockState.getBlock()).toString();
  }

  private static void populateInventory(ClientPlayerEntity player, TelemetrySample sample) {
    Map<String, Integer> counts = new HashMap<>();
    for (int slot = 0; slot < player.getInventory().size(); slot += 1) {
      ItemStack stack = player.getInventory().getStack(slot);
      if (stack.isEmpty()) {
        continue;
      }
      String itemId = Registries.ITEM.getId(stack.getItem()).toString();
      counts.merge(itemId, stack.getCount(), Integer::sum);
    }

    for (Map.Entry<String, Integer> entry : counts.entrySet()) {
      TelemetrySample.InventoryEntrySample inventoryEntry = new TelemetrySample.InventoryEntrySample();
      inventoryEntry.itemId = entry.getKey();
      inventoryEntry.count = entry.getValue();
      sample.inventorySummary.add(inventoryEntry);
    }
  }

  private static double[] floatBufferToColumnMajorArray(FloatBuffer buffer) {
    double[] values = new double[16];
    for (int index = 0; index < 16; index += 1) {
      values[index] = buffer.get(index);
    }
    return values;
  }

  private static void copyMatrix(double[] source, double[] destination) {
    System.arraycopy(source, 0, destination, 0, Math.min(source.length, destination.length));
  }

  private static void copyMatrix(Matrix4f source, double[] destination) {
    FloatBuffer buffer = BufferUtils.createFloatBuffer(16);
    source.get(buffer);
    copyMatrix(floatBufferToColumnMajorArray(buffer), destination);
  }

  private static TelemetryWriter telemetryWriter(MinecraftClient client) {
    if (writer == null) {
      Path runDir = client.runDirectory.toPath();
      writer = new TelemetryWriter(runDir.resolve("auv").resolve("telemetry.jsonl"));
    }
    return writer;
  }
}
