import type { SymbolEntry, PascalType } from '../types';

const SCOPE_BG = ['#EFF6FF', '#F0FDF4', '#FFF7ED', '#FDF4FF'];

function typeLabel(t: PascalType): string {
  if (typeof t === 'string') return t;
  if ('Array' in t) return `Array[${t.Array.low}..${t.Array.high}] of ${typeLabel(t.Array.element)}`;
  return JSON.stringify(t);
}

export function SymbolTable({ entries }: { entries: SymbolEntry[] }) {
  if (entries.length === 0) return <div className="no-errors">No symbols</div>;
  return (
    <table className="sym-table">
      <thead>
        <tr><th>Name</th><th>Kind</th><th>Type</th><th>Scope</th><th>Line</th></tr>
      </thead>
      <tbody>
        {entries.map((e, i) => (
          <tr key={i} style={{ background: SCOPE_BG[e.scope_level % SCOPE_BG.length] }}>
            <td style={{ paddingLeft: e.scope_level * 10 + 4 }}>{e.name}</td>
            <td>{String(e.kind)}</td>
            <td>{typeLabel(e.pascal_type)}</td>
            <td>{e.scope_level}</td>
            <td>{e.line}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
