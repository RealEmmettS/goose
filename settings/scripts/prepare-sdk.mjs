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
console.log('Pinned Native SDK windowless spawn and custom bold-face patches verified.');
await import('./prepare-accessibility.mjs');
