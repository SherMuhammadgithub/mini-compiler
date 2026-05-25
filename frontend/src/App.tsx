import { useState, useCallback, useRef, useEffect } from 'react';
import Editor, { type Monaco, type OnMount } from '@monaco-editor/react';
import { Allotment } from 'allotment';
import 'allotment/dist/style.css';
import { useCompiler } from './hooks/useCompiler';
import { loadWasm } from './hooks/useWasm';
import type {
  CompilerError, TacInstr,
  VmOutput as VmOut, Span,
  FirstFollowOutput, Ll1TableOutput, LrTableOutput,
} from './types';
import { TokenPanel }    from './components/TokenPanel';
import { ErrorPanel }    from './components/ErrorPanel';
import { AstView }       from './components/AstView';
import { SymbolTable }   from './components/SymbolTable';
import { StepMode }      from './components/StepMode';
import { ReportTables }  from './components/ReportTables';
import './App.css';

// ── Sample programs ───────────────────────────────────────────────────────────

const EXAMPLES: Record<string, string> = {
  'Simple (default)':
`program example ( input , output ) ;
var n : integer ;
begin
  n := 5 ;
  if n > 0 then
    write ( n )
  else
    write ( 0 )
end .`,

  'GCD (Aho App. A)':
`program gcd_example ( input , output ) ;
var x , y : integer ;
function gcd ( a , b : integer ) : integer ;
begin
  if b = 0 then gcd := a
  else gcd := gcd ( b , a mod b )
end ;
begin
  read ( x , y ) ;
  write ( gcd ( x , y ) )
end .`,

  'Array Sum':
`program arraysum ( input , output ) ;
var a : array [ 1 .. 10 ] of integer ;
var i , sum : integer ;
begin
  sum := 0 ;
  i := 1 ;
  while i <= 10 do
  begin
    read ( a [ i ] ) ;
    sum := sum + a [ i ] ;
    i := i + 1
  end ;
  write ( sum )
end .`,

  'Factorial (recursive)':
`program factorial ( input , output ) ;
var n : integer ;
function fact ( k : integer ) : integer ;
begin
  if k = 0 then fact := 1
  else fact := k * fact ( k - 1 )
end ;
begin
  read ( n ) ;
  write ( fact ( n ) )
end .`,
};

const DEFAULT_SOURCE = EXAMPLES['Simple (default)'];

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

// ── App root ──────────────────────────────────────────────────────────────────

export default function App() {
  const [source, setSource]         = useState(DEFAULT_SOURCE);
  const [progInput, setProgInput]   = useState('');
  const [dark, setDark]             = useState(true);
  const [activeTab, setActiveTab]   = useState<'tokens'|'ir'|'vm'|'ast'|'reports'>('tokens');
  const [stepMode, setStepMode]     = useState(false);
  const [stepStage, setStepStage]   = useState(0);
  const [firstFollow, setFF]        = useState<FirstFollowOutput | null>(null);
  const [ll1Table,   setLl1]        = useState<Ll1TableOutput    | null>(null);
  const [lrTables,   setLr]         = useState<LrTableOutput     | null>(null);
  const editorRef  = useRef<Parameters<OnMount>[0] | null>(null);
  const monacoRef  = useRef<Monaco | null>(null);
  const decorIds   = useRef<string[]>([]);

  // Load grammar tables once — they depend only on the grammar, not on user input.
  useEffect(() => {
    loadWasm().then(wasm => {
      setFF(JSON.parse(wasm.get_first_follow()));
      setLl1(JSON.parse(wasm.get_ll1_table()));
      setLr(JSON.parse(wasm.get_lr_tables()));
    });
  }, []);

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

  const highlightSpan = (span: Span) => {
    const editor = editorRef.current;
    const monaco = monacoRef.current;
    if (!editor || !monaco) return;
    const range = { startLineNumber: span.line, startColumn: span.column, endLineNumber: span.line, endColumn: span.column + span.length };
    editor.setSelection(range);
    editor.revealRangeInCenter(range);
    editor.focus();
  };

  const toggleDark = () => {
    const next = !dark;
    setDark(next);
    monacoRef.current?.editor.setTheme(next ? 'pascal-dark' : 'pascal-light');
  };

  // Stage → tab mapping for demo step mode
  const STAGE_TABS = ['tokens','ast','reports','reports','tokens','ast','ir','vm'] as const;
  const goStep = (next: number) => {
    setStepStage(next);
    setActiveTab(STAGE_TABS[next]);
  };
  const toggleStep = () => {
    const entering = !stepMode;
    setStepMode(entering);
    if (entering) { setStepStage(0); setActiveTab('tokens'); }
  };

  return (
    <div className={`app ${dark ? 'dark' : 'light'}`}>
      <header className="toolbar">
        <span className="brand">Pascal Compiler</span>
        <select
          className="example-select"
          defaultValue=""
          onChange={e => { if (e.target.value) { setSource(EXAMPLES[e.target.value]); e.target.value = ''; } }}
        >
          <option value="" disabled>Examples…</option>
          {Object.keys(EXAMPLES).map(k => <option key={k} value={k}>{k}</option>)}
        </select>
        {outputs.loading && <span className="loading-dot">●</span>}
        <div className="tabs">
          {stepMode
            ? <StepMode active={stepStage} onPrev={() => goStep(stepStage - 1)} onNext={() => goStep(stepStage + 1)} />
            : (['tokens','ir','vm','ast','reports'] as const).map(t => (
                <button key={t} className={activeTab === t ? 'active' : ''} onClick={() => setActiveTab(t)}>
                  {{ tokens: 'Tokens', ir: 'TAC / IR', vm: 'VM Output', ast: 'AST', reports: 'Reports' }[t]}
                </button>
              ))
          }
        </div>
        <button className="theme-btn" onClick={toggleStep}>
          {stepMode ? 'Exit Demo' : 'Demo'}
        </button>
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
                {activeTab === 'ast' && (
                  <div className="panel">
                    <div className="panel-title">AST — click a node to jump to source</div>
                    <div style={{ flex: 1, overflow: 'hidden' }}>
                      <AstView ast={outputs.rdAst?.ast ?? null} onNodeClick={highlightSpan} />
                    </div>
                  </div>
                )}
                {activeTab === 'reports' && (
                  <Panel title="Report Tables — FIRST/FOLLOW · LL(1) · LR">
                    <ReportTables ff={firstFollow} ll1={ll1Table} lr={lrTables} />
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
                      <SymbolTable entries={outputs.semantic?.symbol_snapshot ?? []} />
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
