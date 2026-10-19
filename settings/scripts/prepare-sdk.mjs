// Native SDK 0.5.4 spawns console-subsystem children with Zig's visible-console
// default on Windows. Keep Honk300's settings service windowless (ADR 0041).
// This applies only to our locked local dependency; global toolkits are untouched.
import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
function patch(relative, originalHash, before, after) {
  const file = fileURLToPath(new URL('../node_modules/@native-sdk/cli/src/' + relative, import.meta.url));
  const source = readFileSync(file, 'utf8');
  const original = source.includes(after) ? source.replace(after, before) : source;
  if (createHash('sha256').update(original).digest('hex') !== originalHash || original.split(before).length !== 2) {
    throw new Error('Unexpected Native SDK 0.5.4 source; refusing patch: ' + relative);
  }
  if (source === original) writeFileSync(file, original.replace(before, after));
}
patch('runtime/effects.zig', 'ffa0af958bded1489211f0c51f36327213f65837fc42437e9a1efa01d6a2da7b',
  '                .argv = ctx.argv(),',
  '                .argv = ctx.argv(),\n                .create_no_window = true, // Honk300: settings services never open a console.');
// 0.5.4 registers custom TTFs but silently uses their base face for bold spans.
// Resolve an explicitly registered bold companion through the same measurement
// and painting seam. Unmodified SDK themes retain their original behavior.
patch('primitives/canvas/tokens.zig', '90820897f491d1fad04671ad3ffbfda8cbe0b2fc7804b55f28a21bfd03b6ddae',
  'pub const TypographyTokens = struct {',
  'pub const TypographyTokens = struct {\n    bold_font_id: ?FontId = null, // Honk300: registered bold companion.');
patch('primitives/canvas/text_spans.zig', 'c4c799324ee6f5f5ca31fe3a370eac15f04a8be26d6878bde210b20dc4915ceb',
  '    if (span.monospace) return typography.mono_font_id;',
  '    if (span.monospace) return typography.mono_font_id;\n    if (span.weight == .bold and !span.italic) {\n        if (typography.bold_font_id) |id| return id;\n    }');
// Wheel inputs that clamp or use reduced motion have no kinetic frame to repair
// their accessibility geometry. Request the existing coalesced post-present
// publication when wheel input changes the layout; idle frames remain unchanged.
patch('runtime/canvas_widget_events.zig', '6939934e1a3ea915ac303978cca0a5dd615ae0ec6ec987708a5d66e7a7ad6696',
  '            _ = try runtime_canvas_widget_display.RuntimeCanvasWidgetDisplay(Runtime).refreshCanvasWidgetDisplayListIfOwnedSkippingAccessibility(self, index);',
  '            self.views[index].canvas_widget_wheel_timestamp_ns = self.views[index].gpu_input_timestamp_ns;\n            _ = try runtime_canvas_widget_display.RuntimeCanvasWidgetDisplay(Runtime).refreshCanvasWidgetDisplayListIfOwned(self, index); // Honk300: publish wheel-only geometry.');
patch('runtime/view.zig', '6752f653f548ad03d5c9a764573f25a7d401cb98bedfb96372b4d2692647d296',
  '    gpu_timestamp_ns: u64 = 0,',
  '    gpu_timestamp_ns: u64 = 0,\n    canvas_widget_wheel_timestamp_ns: u64 = 0, // Honk300: only changed wheel input rebases decay.');
// The host interval is nominal cadence; delayed completion timestamps are the
// scroll clock. Keep cadence/latency metadata unchanged and rebase after input.
const frameClockBefore = `                self.views[index].gpu_timestamp_ns = frame_event.timestamp_ns;
                self.views[index].recordGpuSurfaceFrameInterval(frame_event.frame_interval_ns);
                self.views[index].recordGpuSurfaceFirstFrameLatency(frame_event.timestamp_ns);
                // Host-stamped packet decode/draw splits ride the frame
                // event (zero on completion-only frames): feed the frame
                // profile's host stages while profiling is on.
                if (frame_event.packet_decode_ns > 0) self.frame_profile.recordNs(.host_decode, frame_event.packet_decode_ns);
                if (frame_event.packet_draw_ns > 0) self.frame_profile.recordNs(.host_draw, frame_event.packet_draw_ns);
                try CanvasWidgetDisplayMethods().advanceCanvasWidgetKineticScrollForFrame(self, index, frame_event.frame_interval_ns, had_pending_input);`;
const frameClockAfter = frameClockBefore.replace(
  '                self.views[index].gpu_timestamp_ns = frame_event.timestamp_ns;',
  `                const previous_scroll_ns = @max(self.views[index].gpu_timestamp_ns, self.views[index].canvas_widget_wheel_timestamp_ns);
                const kinetic_interval_ns = if (previous_scroll_ns > 0 and frame_event.timestamp_ns > previous_scroll_ns)
                    frame_event.timestamp_ns - previous_scroll_ns
                else
                    frame_event.frame_interval_ns;
                self.views[index].gpu_timestamp_ns = frame_event.timestamp_ns;`
).replace('self, index, frame_event.frame_interval_ns, had_pending_input', 'self, index, kinetic_interval_ns, had_pending_input');
patch('runtime/gpu_surface_events.zig', '9ed40fbfb87ec9ad5011815e9d5ab760b8aff270516a2f77eec32e2a66acc439', frameClockBefore, frameClockAfter);
// Integrate a delayed frame in small bounded steps, then publish geometry once.
// After a long pause, old wheel momentum expires instead of making a large jump.
patch('runtime/view_widget_scroll.zig', '401e2604ca2639f7686a41a9ea0a83245592f0e50c42b63338569fa7ceb7b3ab',
  '                const next = current.stepKinetic(dt_ms, physics);',
  `                var next = current;
                var remaining_ms = @min(@max(dt_ms, 0), 1000);
                var steps: usize = 0;
                while (remaining_ms > 0 and steps < 64 and next.needsKineticStep(physics)) : (steps += 1) {
                    const step_ms = @min(remaining_ms, 1000.0 / 60.0);
                    next = next.stepKinetic(step_ms, physics);
                    remaining_ms -= step_ms;
                }
                if (dt_ms > 1000) next.velocity = 0;`);
console.log('Pinned Native SDK windowless spawn, bold-face, wheel accessibility and elapsed-scroll patches verified.');
await import('./prepare-accessibility.mjs');
