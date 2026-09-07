// Apply only reviewed, hash-pinned source edits to the local SDK dependency.
import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
const recipes = JSON.parse(readFileSync(new URL('./sdk-accessibility-patches.json', import.meta.url), 'utf8'));
for (const { file, sha256, edits } of recipes) {
  const path = fileURLToPath(new URL('../node_modules/@native-sdk/cli/src/' + file, import.meta.url));
  const current = readFileSync(path, 'utf8');
  let original = current;
  for (const { before, after } of [...edits].reverse()) {
    if (original.includes(after)) original = original.replace(after, before);
  }
  if (createHash('sha256').update(original).digest('hex') !== sha256) {
    throw new Error('Unexpected Native SDK accessibility source: ' + file);
  }
  let patched = original;
  for (const { before, after } of edits) {
    if (patched.split(before).length !== 2) throw new Error('Ambiguous accessibility patch: ' + file);
    patched = patched.replace(before, after);
  }
  if (current !== patched) writeFileSync(path, patched);
}
console.log('Pinned native accessibility adapters verified.');
