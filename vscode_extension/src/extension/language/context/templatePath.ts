/** Template path under `path =` / `c:template({ path })` / long-string template refs. */
export function matchTemplatePath(
  line: string,
  character: number
): string | undefined {
  const match =
    /(?:path|template)\s*=\s*['"]([^'"]+\.template)['"]/.exec(line) ??
    /template\s*\(\s*\{\s*path\s*=\s*['"]([^'"]+)['"]/.exec(line);
  if (!match || match.index === undefined) {
    return undefined;
  }
  const path = match[1];
  const start = match.index + match[0].indexOf(path);
  const end = start + path.length;
  if (character >= start && character <= end) {
    return path;
  }
  return undefined;
}
