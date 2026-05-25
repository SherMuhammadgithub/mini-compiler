import type { FirstFollowOutput, Ll1TableOutput, LrTableOutput } from '../types';

const TH: React.CSSProperties = { padding: '3px 8px', background: '#1E2530', color: '#C9D1D9', fontWeight: 600, whiteSpace: 'nowrap' };
const TD: React.CSSProperties = { padding: '2px 8px', borderBottom: '1px solid #30363D' };

export function ReportTables({ ff, ll1, lr }: {
  ff:  FirstFollowOutput | null;
  ll1: Ll1TableOutput    | null;
  lr:  LrTableOutput     | null;
}) {
  return (
    <div style={{ fontFamily: 'ui-monospace, Consolas, monospace', fontSize: 12, overflowY: 'auto', padding: 12, color: 'var(--text)' }}>

      {/* ── FIRST & FOLLOW ── */}
      <h3 style={{ margin: '0 0 6px' }}>FIRST and FOLLOW Sets</h3>
      {ff && (
        <table style={{ borderCollapse: 'collapse', width: '100%', marginBottom: 20 }}>
          <thead><tr><th style={TH}>Non-Terminal</th><th style={TH}>FIRST</th><th style={TH}>FOLLOW</th></tr></thead>
          <tbody>
            {ff.rows.map((row, i) => (
              <tr key={i} style={{ background: i % 2 ? 'var(--bg2)' : 'var(--bg)' }}>
                <td style={{ ...TD, fontWeight: 600 }}>{row.non_terminal}</td>
                <td style={TD}>{'{' + row.first.join(', ') + '}'}</td>
                <td style={TD}>{'{' + row.follow.join(', ') + '}'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {/* ── LL(1) PARSING TABLE ── */}
      <h3 style={{ margin: '0 0 6px' }}>LL(1) Parsing Table M[A, a]</h3>
      {ll1 && (
        <div style={{ overflowX: 'auto', marginBottom: 20 }}>
          <table style={{ borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={TH}>Non-Terminal</th>
                {ll1.terminals.map(t => <th key={t} style={TH}>{t}</th>)}
              </tr>
            </thead>
            <tbody>
              {ll1.rows.map((row, i) => (
                <tr key={i} style={{ background: i % 2 ? 'var(--bg2)' : 'var(--bg)' }}>
                  <td style={{ ...TD, fontWeight: 600 }}>{row.non_terminal}</td>
                  {ll1.terminals.map(t => (
                    <td key={t} style={{ ...TD, color: row.cells[t] ? 'var(--text)' : 'var(--text-dim)', fontSize: 11 }}>
                      {row.cells[t] ?? ''}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* ── LR ACTION TABLE ── */}
      <h3 style={{ margin: '0 0 6px' }}>LR Action Table</h3>
      {lr && (
        <div style={{ overflowX: 'auto', marginBottom: 20 }}>
          <table style={{ borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={TH}>State</th>
                {lr.terminals.map(t => <th key={t} style={TH}>{t}</th>)}
              </tr>
            </thead>
            <tbody>
              {lr.action_rows.map((row, i) => (
                <tr key={i} style={{ background: i % 2 ? 'var(--bg2)' : 'var(--bg)' }}>
                  <td style={{ ...TD, fontWeight: 600 }}>{i}</td>
                  {lr.terminals.map(t => (
                    <td key={t} style={{ ...TD, color: row[t] === 'acc' ? '#3FB950' : row[t]?.startsWith('s') ? '#60A5FA' : 'var(--text)', fontSize: 11 }}>
                      {row[t] ?? ''}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* ── LR GOTO TABLE ── */}
      <h3 style={{ margin: '0 0 6px' }}>LR Goto Table</h3>
      {lr && (
        <div style={{ overflowX: 'auto', marginBottom: 20 }}>
          <table style={{ borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={TH}>State</th>
                {lr.non_terminals.map(n => <th key={n} style={TH}>{n}</th>)}
              </tr>
            </thead>
            <tbody>
              {lr.goto_rows.map((row, i) => (
                <tr key={i} style={{ background: i % 2 ? 'var(--bg2)' : 'var(--bg)' }}>
                  <td style={{ ...TD, fontWeight: 600 }}>{i}</td>
                  {lr.non_terminals.map(n => (
                    <td key={n} style={{ ...TD, color: row[n] != null ? 'var(--text)' : 'var(--text-dim)', fontSize: 11 }}>
                      {row[n] ?? ''}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <button
        onClick={() => window.print()}
        style={{ padding: '6px 16px', background: 'var(--accent)', color: '#fff', border: 'none', borderRadius: 6, cursor: 'pointer', marginBottom: 12 }}
      >
        Print / Save for Report
      </button>
    </div>
  );
}
