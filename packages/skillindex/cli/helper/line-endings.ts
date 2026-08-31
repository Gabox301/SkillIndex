// Shared helper for normalizing line endings across scripts.
// Converts CRLF and lone CR sequences to LF so that hashes computed on
// downloaded/registry files are stable regardless of the platform.

export function normalizeLineEndings(buf: Buffer): Buffer {
  const str = buf.toString('utf-8');
  if (!str.includes('\r')) return buf;
  return Buffer.from(str.replace(/\r\n/g, '\n').replace(/\r/g, '\n'), 'utf-8');
}
