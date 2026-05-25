import type { CompilerError } from '../types';

export function ErrorPanel({
  errors,
  onJump,
}: {
  errors: CompilerError[];
  onJump: (line: number, col: number) => void;
}) {
  if (errors.length === 0) {
    return <div className="no-errors">✓ No errors</div>;
  }
  return (
    <div className="error-list">
      {errors.map((e, i) => (
        <div
          key={i}
          className={`error-row ${e.severity}`}
          onClick={() => onJump(e.line, e.column)}
        >
          <span className="err-stage">[{e.stage}]</span>
          <span className="err-loc">L{e.line}:{e.column}</span>
          <span className="err-msg">{e.message}</span>
        </div>
      ))}
    </div>
  );
}
