export const RESULT_BEGIN = '__CODEMOD_RESULT_BEGIN__';
export const RESULT_END = '__CODEMOD_RESULT_END__';

export function extractResult(output) {
  const begin = output.indexOf(RESULT_BEGIN);
  const end = output.indexOf(RESULT_END);
  if (begin === -1 || end === -1 || end < begin) return undefined;
  return output.slice(begin + RESULT_BEGIN.length, end).trim();
}
