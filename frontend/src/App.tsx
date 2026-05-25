import { useState, useCallback, useRef } from 'react';
import Editor, { type Monaco, type OnMount } from '@monaco-editor/react';
import { Allotment } from 'allotment';
import 'allotment/dist/style.css';
import { useCompiler } from './hooks/useCompiler';
import type {
  CompilerError, TacInstr, SymbolEntry,
  VmOutput as VmOut,
} from './types';
import { TokenPanel } from './components/TokenPanel';
import { ErrorPanel } from './components/ErrorPanel';
import './App.css';

// ── Default program ───────────────────────────────────────────────────────────

const DEFAULT_SOURCE = `program example ( input , output ) ;
var n : integer ;
begin
  n := 5 ;
  if n > 0 then
    write ( n )
  else
    write ( 0 )
end .`;

// ── Monaco Pascal language registration ───────────────────────────────────────

function registerPascal(monaco: Monaco) {
  if (monaco.languages.getLanguages().some(l => l.id === 'pascal')) return;
  monaco.languages.register({ id: 'pascal' });
  monaco.languages.setMonarchTokensProvider('pascal', {
    defaultToken: '',
    tokenPostfix: '.pascal',
    keywords: [
      'program','var','array','of','integer','real','function','procedure',
      'begin','end','if','then','else','while','do','not','and','or','div','mod',
    ],
    operators: [':=','=','<>','<','<=','>','>=','+','-','*','/'],
    symbols: /[=><!~?:&|+\-*/^%]+/,
    tokenizer: {
      root: [
        [/[a-zA-Z_]\w*/, { cases: { '@keywords': 'keyword', '@default': 'identifier' } }],
        [/\d+\.\d*/, 'number.float'],
        [/\d+/, 'number'],
        [/\{[^}]*\}/, 'comment'],
        [/'[^']*'/, 'string'],
        [/[;:,.\[\]()]/, 'delimiter'],
        [/@symbols/, { cases: { '@operators': 'operator', '@default': '' } }],
        [/[ \t\r\n]+/, 'white'],
      ],
    },
  });
  monaco.editor.defineTheme('pascal-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'keyword',      foreground: 'C084FC', fontStyle: 'bold' },
      { token: 'identifier',   foreground: '93C5FD' },
      { token: 'number',       foreground: '6EE7B7' },
      { token: 'number.float', foreground: '6EE7B7' },
      { token: 'string',       foreground: 'FCD34D' },
      { token: 'comment',      foreground: '6B7280', fontStyle: 'italic' },
      { token: 'operator',     foreground: 'FB923C' },
      { token: 'delimiter',    foreground: '9CA3AF' },
    ],
    colors: { 'editor.background': '#0F1117' },
  });
  monaco.editor.defineTheme('pascal-light', {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'keyword',    foreground: '7C3AED', fontStyle: 'bold' },
      { token: 'identifier', foreground: '1D4ED8' },
      { token: 'number',     foreground: '065F46' },
      { token: 'string',     foreground: '92400E' },
      { token: 'comment',    foreground: '6B7280', fontStyle: 'italic' },
      { token: 'operator',   foreground: 'B45309' },
    ],
    colors: {},
  });
}

// ── Error decorations ─────────────────────────────────────────────────────────

function applyDecorations(
  editor: Parameters<OnMount>[0],
  monaco: Monaco,
  errors: CompilerError[],
  decorIds: React.MutableRefObject<string[]>,
) {
  const newIds = editor.deltaDecorations(
    decorIds.current,
    errors.map(e => ({
      range: new monaco.Range(e.line, e.column, e.line, e.column + Math.max(e.length, 1)),
      options: {
        inlineClassName: 'error-underline',
        glyphMarginClassName: 'error-gutter',
        hoverMessage: { value: `**[${e.stage}]** ${e.message}` },
      },
    })),
  );
  decorIds.current = newIds;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function allErrors(outputs: ReturnType<typeof useCompiler>): CompilerError[] {
  return [outputs.lexer?.errors, outputs.rdAst?.errors, outputs.semantic?.errors]
    .flatMap(l => l ?? []);
}

function Panel({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="panel">
      <div className="panel-title">{title}</div>
      <div className="panel-body">{children}</div>
    </div>
  );
}

// ── Sub-components ────────────────────────────────────────────────────────────

function TacList({ instrs }: { instrs: TacInstr[] }) {
  return (
    <div className="tac-list">
      {instrs.map((ins, i) => {
        const args = [ins.arg1, ins.arg2, ins.result]
          .filter(Boolean)
          .map(a => {
            if (!a) return '';
            const [k, v] = Object.entries(a)[0];
            return `${k}(${v})`;
          })
          .join('  ');
        return (
          <div key={i} className="tac-row">
            <span className="tac-idx">{i}</span>
            <span className="tac-op">{String(ins.op)}</span>
            <span className="tac-args">{args}</span>
          </div>
        );
      })}
    </div>
  );
}

function VmOutputPanel({ out }: { out: VmOut | null }) {
  if (!out) return <div className="vm-empty">Run a valid program to see output</div>;
  return (
    <div className="vm-out">
      <div className="vm-stdout">
        {out.stdout.length === 0
          ? <span className="vm-no-out">(no output)</span>
          : out.stdout.map((line, i) => <div key={i} className="vm-line">{line || ' '}</div>)
        }
      </div>
      <div className="vm-meta">Steps: {out.step_count} · Halted: {out.halted ? 'yes' : 'no'}</div>
    </div>
  );
}

const SCOPE_BG = ['#EFF6FF','#F0FDF4','#FFF7ED','#FDF4FF'];

function SymbolList({ entries }: { entries: SymbolEntry[] }) {
  if (entries.length === 0) return <div className="no-errors">No symbols</div>;
  return (
    <table className="sym-table">
      <thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Scope</th></tr></thead>
      <tbody>
        {entries.map((e, i) => (
          <tr key={i} style={{ background: SCOPE_BG[e.scope_level % SCOPE_BG.length] }}>
            <td style={{ paddingLeft: e.scope_level * 10 + 4 }}>{e.name}</td>
            <td>{String(e.kind)}</td>
            <td>{typeof e.pascal_type === 'string' ? e.pascal_type : JSON.stringify(e.pascal_type)}</td>
            <td>{e.scope_level}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

// ── App root ──────────────────────────────────────────────────────────────────

export default function App() {
  const [source, setSource] = useState(DEFAULT_SOURCE);
  const [progInput, setProgInput] = useState('');
  const [dark, setDark] = useState(true);
  const [activeTab, setActiveTab] = useState<'tokens'|'ir'|'vm'>('tokens');
  const editorRef  = useRef<Parameters<OnMount>[0] | null>(null);
  const monacoRef  = useRef<Monaco | null>(null);
  const decorIds   = useRef<string[]>([]);

  const outputs = useCompiler(source, progInput);
  const errors  = allErrors(outputs);

  // Re-apply Monaco error decorations on every render where editor is mounted.
  if (editorRef.current && monacoRef.current) {
    applyDecorations(editorRef.current, monacoRef.current, errors, decorIds);
  }

  const handleMount: OnMount = useCallback((editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;
    registerPascal(monaco);
    monaco.editor.setTheme(dark ? 'pascal-dark' : 'pascal-light');
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const jumpTo = (line: number, col: number) => {
    editorRef.current?.revealLineInCenter(line);
    editorRef.current?.setPosition({ lineNumber: line, column: col });
    editorRef.current?.focus();
  };

  const toggleDark = () => {
    const next = !dark;
    setDark(next);
    monacoRef.current?.editor.setTheme(next ? 'pascal-dark' : 'pascal-light');
  };

  return (
    <div className={`app ${dark ? 'dark' : 'light'}`}>
      <header className="toolbar">
        <span className="brand">Pascal Compiler</span>
        {outputs.loading && <span className="loading-dot">●</span>}
        <div className="tabs">
          {(['tokens','ir','vm'] as const).map(t => (
            <button key={t} className={activeTab === t ? 'active' : ''} onClick={() => setActiveTab(t)}>
              {t === 'tokens' ? 'Tokens' : t === 'ir' ? 'TAC / IR' : 'VM Output'}
            </button>
          ))}
        </div>
        <button className="theme-btn" onClick={toggleDark} aria-label="Toggle theme">
          {dark ? '☀' : '☾'}
        </button>
      </header>

      <div className="main">
        <Allotment>
          <Allotment.Pane minSize={280}>
            <Editor
              language="pascal"
              value={source}
              theme={dark ? 'pascal-dark' : 'pascal-light'}
              onChange={v => setSource(v ?? '')}
              onMount={handleMount}
              options={{
                fontSize: 14, minimap: { enabled: false },
                glyphMargin: true, lineNumbers: 'on',
                scrollBeyondLastLine: false, wordWrap: 'on',
              }}
            />
          </Allotment.Pane>

          <Allotment.Pane minSize={280}>
            <Allotment vertical>
              <Allotment.Pane minSize={120}>
                {activeTab === 'tokens' && (
                  <Panel title={`Tokens (${outputs.lexer?.tokens?.length ?? 0})`}>
                    <TokenPanel tokens={outputs.lexer?.tokens ?? []} />
                  </Panel>
                )}
                {activeTab === 'ir' && (
                  <Panel title={`TAC Instructions (${outputs.ir?.instructions?.length ?? 0})`}>
                    <TacList instrs={outputs.ir?.instructions ?? []} />
                  </Panel>
                )}
                {activeTab === 'vm' && (
                  <Panel title="VM Execution">
                    <div className="vm-io">
                      <label>Program input:</label>
                      <input
                        className="prog-input"
                        value={progInput}
                        onChange={e => setProgInput(e.target.value)}
                        placeholder="whitespace-separated values"
                      />
                    </div>
                    <VmOutputPanel out={outputs.vm} />
                  </Panel>
                )}
              </Allotment.Pane>

              <Allotment.Pane minSize={80}>
                <Allotment>
                  <Allotment.Pane>
                    <Panel title={`Errors (${errors.length})`}>
                      <ErrorPanel errors={errors} onJump={jumpTo} />
                    </Panel>
                  </Allotment.Pane>
                  <Allotment.Pane>
                    <Panel title={`Symbols (${outputs.semantic?.symbol_snapshot?.length ?? 0})`}>
                      <SymbolList entries={outputs.semantic?.symbol_snapshot ?? []} />
                    </Panel>
                  </Allotment.Pane>
                </Allotment>
              </Allotment.Pane>
            </Allotment>
          </Allotment.Pane>
        </Allotment>
      </div>
    </div>
  );
}
