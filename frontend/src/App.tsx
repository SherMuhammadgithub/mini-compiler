import { useState, useCallback, useRef, useEffect } from 'react';
import Editor, { type Monaco, type OnMount } from '@monaco-editor/react';
import { Allotment } from 'allotment';
import 'allotment/dist/style.css';
import {
  Moon, Sun, Loader2, Play, Hash, Code2,
  Terminal, GitBranch, BarChart3, AlertCircle, Package,
} from 'lucide-react';
import { useCompiler } from './hooks/useCompiler';
import { loadWasm } from './hooks/useWasm';
import type {
  CompilerError, TacInstr,
  VmOutput as VmOut, Span,
  FirstFollowOutput, Ll1TableOutput, LrTableOutput,
} from './types';
import { TokenPanel }   from './components/TokenPanel';
import { ErrorPanel }   from './components/ErrorPanel';
import { AstView }      from './components/AstView';
import { SymbolTable }  from './components/SymbolTable';
import { StepMode }     from './components/StepMode';
import { ReportTables } from './components/ReportTables';
import { ScrollArea }   from './components/ui/scroll-area';
import { TooltipProvider, Tooltip, TooltipTrigger, TooltipContent } from './components/ui/tooltip';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './components/ui/select';
import { cn } from './lib/utils';
// ── Sample programs ───────────────────────────────────────────────────────────

const EXAMPLES: Record<string, string> = {
  'Simple':
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

  'Factorial':
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

const DEFAULT_SOURCE = EXAMPLES['Simple'];

// ── Monaco ────────────────────────────────────────────────────────────────────

function registerPascal(monaco: Monaco) {
  // Always define themes first — defineTheme is idempotent and the editor
  // needs them before it can apply the 'pascal-dark'/'pascal-light' theme prop.
  // Placing this after the early-return guard caused the editor to stay white
  // because Monaco has a built-in 'pascal' language entry, triggering the guard.
  monaco.editor.defineTheme('pascal-dark', {
    base: 'vs-dark', inherit: true,
    rules: [
      { token: 'keyword',      foreground: 'E2E8F0', fontStyle: 'bold' },
      { token: 'identifier',   foreground: '93C5FD' },
      { token: 'number',       foreground: '6EE7B7' },
      { token: 'number.float', foreground: '6EE7B7' },
      { token: 'string',       foreground: 'FCD34D' },
      { token: 'comment',      foreground: '52525b', fontStyle: 'italic' },
      { token: 'operator',     foreground: 'FB923C' },
      { token: 'delimiter',    foreground: '71717a' },
    ],
    colors: {
      'editor.background':              '#0a0a0a',
      'editor.foreground':              '#e4e4e7',
      'editor.lineHighlightBackground': '#1a1a1a',
      'editorGutter.background':        '#0a0a0a',
      'editorLineNumber.foreground':    '#52525b',
      'editorLineNumber.activeForeground': '#a1a1aa',
    },
  });
  monaco.editor.defineTheme('pascal-light', {
    base: 'vs', inherit: true,
    rules: [
      { token: 'keyword',    foreground: '18181b', fontStyle: 'bold' },
      { token: 'identifier', foreground: '1D4ED8' },
      { token: 'number',     foreground: '065F46' },
      { token: 'string',     foreground: '92400E' },
      { token: 'comment',    foreground: '71717a', fontStyle: 'italic' },
      { token: 'operator',   foreground: 'B45309' },
    ],
    colors: {},
  });

  // Only register the custom tokenizer once (skip if already registered)
  if (monaco.languages.getLanguages().some((l: { id: string }) => l.id === 'pascal')) return;
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

// ── Panel ─────────────────────────────────────────────────────────────────────

type PanelIcon = typeof Hash;

function Panel({ title, count, icon: Icon, children }: {
  title: string;
  count?: number;
  icon?: PanelIcon;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col h-full" style={{ background: 'var(--card)' }}>
      <div
        className="flex items-center gap-2 px-3 shrink-0"
        style={{ height: 34, borderBottom: '1px solid var(--border)', background: 'var(--muted)' }}
      >
        {Icon && <Icon style={{ width: 12, height: 12, color: 'var(--muted-foreground)' }} />}
        <span style={{
          fontSize: 10, fontWeight: 700, textTransform: 'uppercase',
          letterSpacing: '0.08em', color: 'var(--muted-foreground)',
        }}>
          {title}
        </span>
        {count !== undefined && (
          <span style={{
            marginLeft: 'auto',
            fontSize: 10, fontFamily: 'monospace',
            padding: '1px 6px', borderRadius: 99,
            background: 'var(--border)', color: 'var(--muted-foreground)',
          }}>
            {count}
          </span>
        )}
      </div>
      <div className="flex-1 overflow-y-auto overflow-x-hidden">
        {children}
      </div>
    </div>
  );
}

// ── TAC list ──────────────────────────────────────────────────────────────────

function TacList({ instrs }: { instrs: TacInstr[] }) {
  if (instrs.length === 0) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 80, color: 'var(--muted-foreground)', fontSize: 12, fontStyle: 'italic' }}>
        No IR yet
      </div>
    );
  }
  return (
    <div style={{ fontFamily: 'monospace', fontSize: 12 }}>
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
          <div key={i} style={{
            display: 'grid', gridTemplateColumns: '28px 110px 1fr',
            gap: 8, padding: '3px 12px',
            borderBottom: '1px solid var(--border)',
          }}>
            <span style={{ color: 'var(--muted-foreground)', textAlign: 'right' }}>{i}</span>
            <span style={{ color: '#60a5fa', fontWeight: 600 }}>{String(ins.op)}</span>
            <span style={{ color: 'var(--muted-foreground)' }}>{args}</span>
          </div>
        );
      })}
    </div>
  );
}

// ── VM output ─────────────────────────────────────────────────────────────────

function VmOutputPanel({ out }: { out: VmOut | null }) {
  if (!out) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 80, color: 'var(--muted-foreground)', fontSize: 12, fontStyle: 'italic' }}>
        Run a valid program to see output
      </div>
    );
  }
  return (
    <div style={{ padding: 12 }}>
      <div style={{
        background: 'var(--muted)', border: '1px solid var(--border)',
        borderRadius: 6, padding: '10px 12px',
        fontFamily: 'monospace', fontSize: 13,
        color: 'var(--foreground)', minHeight: 44, marginBottom: 10,
      }}>
        {out.stdout.length === 0
          ? <span style={{ color: 'var(--muted-foreground)', fontStyle: 'italic' }}>(no output)</span>
          : out.stdout.map((line, i) => <div key={i}>{line || ' '}</div>)
        }
      </div>
      <div style={{ display: 'flex', gap: 16, fontSize: 11, fontFamily: 'monospace', color: 'var(--muted-foreground)' }}>
        <span>Steps: <strong style={{ color: 'var(--foreground)' }}>{out.step_count}</strong></span>
        <span>Halted: <strong style={{ color: out.halted ? '#10b981' : '#f87171' }}>
          {out.halted ? 'yes' : 'no'}
        </strong></span>
      </div>
    </div>
  );
}

// ── App ───────────────────────────────────────────────────────────────────────

const TABS = ['tokens', 'ir', 'vm', 'ast', 'reports'] as const;
type Tab = typeof TABS[number];
const TAB_LABELS: Record<Tab, string> = {
  tokens: 'Tokens', ir: 'TAC / IR', vm: 'VM', ast: 'AST', reports: 'Reports',
};
const TAB_ICONS: Record<Tab, PanelIcon> = {
  tokens: Hash, ir: Code2, vm: Terminal, ast: GitBranch, reports: BarChart3,
};
const STAGE_TABS = ['tokens','ast','reports','reports','tokens','ast','ir','vm'] as const;

export default function App() {
  const [source, setSource]       = useState(DEFAULT_SOURCE);
  const [progInput, setProgInput] = useState('');
  const [dark, setDark]           = useState(true);
  const [activeTab, setActiveTab] = useState<Tab>('tokens');
  const [stepMode, setStepMode]   = useState(false);
  const [stepStage, setStepStage] = useState(0);
  const [exKey, setExKey]         = useState(0);
  const [firstFollow, setFF]      = useState<FirstFollowOutput | null>(null);
  const [ll1Table,   setLl1]      = useState<Ll1TableOutput    | null>(null);
  const [lrTables,   setLr]       = useState<LrTableOutput     | null>(null);
  const editorRef = useRef<Parameters<OnMount>[0] | null>(null);
  const monacoRef = useRef<Monaco | null>(null);
  const decorIds  = useRef<string[]>([]);

  // Apply dark class to <html> so ALL elements (including Allotment panes) inherit it
  useEffect(() => {
    document.documentElement.classList.toggle('dark', dark);
  }, [dark]);

  useEffect(() => {
    loadWasm().then(wasm => {
      setFF(JSON.parse(wasm.get_first_follow()));
      setLl1(JSON.parse(wasm.get_ll1_table()));
      setLr(JSON.parse(wasm.get_lr_tables()));
    });
  }, []);

  const outputs = useCompiler(source, progInput);
  const errors  = allErrors(outputs);

  if (editorRef.current && monacoRef.current) {
    applyDecorations(editorRef.current, monacoRef.current, errors, decorIds);
  }

  const handleBeforeMount = useCallback((monaco: Monaco) => {
    registerPascal(monaco);
  }, []);

  const handleMount: OnMount = useCallback((editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;
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
    const range = {
      startLineNumber: span.line, startColumn: span.column,
      endLineNumber: span.line,   endColumn: span.column + span.length,
    };
    editor.setSelection(range);
    editor.revealRangeInCenter(range);
    editor.focus();
  };

  const toggleDark = () => {
    const next = !dark;
    setDark(next);
    monacoRef.current?.editor.setTheme(next ? 'pascal-dark' : 'pascal-light');
  };

  const goStep = (next: number) => { setStepStage(next); setActiveTab(STAGE_TABS[next] as Tab); };
  const toggleStep = () => {
    const entering = !stepMode;
    setStepMode(entering);
    if (entering) { setStepStage(0); setActiveTab('tokens'); }
  };

  return (
    <TooltipProvider delayDuration={400}>
      <div style={{
        display: 'flex', flexDirection: 'column', height: '100vh', overflow: 'hidden',
        background: 'var(--background)', color: 'var(--foreground)', fontSize: 13,
      }}>

        {/* ── Toolbar ─────────────────────────────────────────────────────── */}
        <header style={{
          display: 'flex', alignItems: 'center', gap: 0,
          height: 44, paddingLeft: 16, paddingRight: 16, flexShrink: 0,
          background: 'var(--card)', borderBottom: '1px solid var(--border)',
        }}>
          {/* Brand */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginRight: 16 }}>
            <div style={{
              width: 22, height: 22, borderRadius: 5,
              background: 'var(--primary)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
            }}>
              <Code2 style={{ width: 12, height: 12, color: 'var(--primary-foreground)' }} />
            </div>
            <span style={{ fontWeight: 700, fontSize: 13, letterSpacing: '-0.01em' }}>
              Pascal Compiler
            </span>
          </div>

          {/* Examples */}
          <Select key={exKey} onValueChange={v => { setSource(EXAMPLES[v]); setExKey(k => k + 1); }}>
            <SelectTrigger className="h-7 w-36 mr-3 shrink-0">
              <SelectValue placeholder="Examples…" />
            </SelectTrigger>
            <SelectContent>
              {Object.keys(EXAMPLES).map(k => (
                <SelectItem key={k} value={k}>{k}</SelectItem>
              ))}
            </SelectContent>
          </Select>

          {outputs.loading && (
            <Loader2 style={{ width: 13, height: 13, color: 'var(--muted-foreground)', marginRight: 8 }}
              className="animate-spin" />
          )}

          {/* Underline tabs */}
          <div style={{ display: 'flex', flex: 1, height: '100%', gap: 0, marginRight: 8 }}>
            {!stepMode && TABS.map(t => {
              const Icon = TAB_ICONS[t];
              const isActive = activeTab === t;
              return (
                <button
                  key={t}
                  onClick={() => setActiveTab(t)}
                  style={{
                    display: 'flex', alignItems: 'center', gap: 5,
                    height: '100%', padding: '0 12px',
                    background: 'transparent', border: 'none',
                    borderBottom: isActive ? '2px solid var(--primary)' : '2px solid transparent',
                    color: isActive ? 'var(--foreground)' : 'var(--muted-foreground)',
                    fontSize: 12, fontWeight: isActive ? 600 : 400,
                    cursor: 'pointer', transition: 'all 0.15s', userSelect: 'none',
                  }}
                >
                  <Icon style={{ width: 11, height: 11 }} />
                  {TAB_LABELS[t]}
                </button>
              );
            })}
          </div>

          {/* Demo button */}
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={toggleStep}
                style={{ fontFamily: 'inherit' }}
                className={cn(
                  'flex items-center gap-2 h-8 px-4 rounded-md border text-[12px] font-semibold',
                  'tracking-wide cursor-pointer transition-all duration-150 mr-1.5 shrink-0',
                  stepMode
                    ? 'border-zinc-500 bg-zinc-800 text-white hover:bg-zinc-700'
                    : 'border-zinc-600 bg-zinc-900 text-zinc-300 hover:border-zinc-400 hover:text-white hover:bg-zinc-800',
                )}
              >
                <Play className={cn('h-3 w-3', stepMode && 'text-emerald-400')} />
                {stepMode ? 'Exit' : 'Demo'}
              </button>
            </TooltipTrigger>
            <TooltipContent>Step through each compiler stage</TooltipContent>
          </Tooltip>

          {/* Theme toggle */}
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={toggleDark}
                style={{ fontFamily: 'inherit' }}
                className="flex items-center justify-center h-8 w-8 rounded-md border border-zinc-600 bg-zinc-900 text-zinc-400 hover:border-zinc-400 hover:text-white hover:bg-zinc-800 cursor-pointer transition-all duration-150 shrink-0"
                aria-label="Toggle theme"
              >
                {dark ? <Sun className="h-3.5 w-3.5" /> : <Moon className="h-3.5 w-3.5" />}
              </button>
            </TooltipTrigger>
            <TooltipContent>{dark ? 'Light mode' : 'Dark mode'}</TooltipContent>
          </Tooltip>
        </header>

        {/* ── Main split ────────────────────────────────────────────────── */}
        <div style={{ flex: 1, overflow: 'hidden', background: 'var(--background)' }}>
          <Allotment>
            <Allotment.Pane minSize={280}>
              <Editor
                language="pascal"
                value={source}
                theme={dark ? 'pascal-dark' : 'pascal-light'}
                beforeMount={handleBeforeMount}
                onChange={v => setSource(v ?? '')}
                onMount={handleMount}
                options={{
                  fontSize: 14, fontFamily: "'JetBrains Mono', 'Cascadia Code', 'Fira Code', ui-monospace, monospace",
                  minimap: { enabled: false },
                  glyphMargin: true, lineNumbers: 'on',
                  scrollBeyondLastLine: false, wordWrap: 'on',
                  padding: { top: 12 },
                  renderLineHighlight: 'gutter',
                }}
              />
            </Allotment.Pane>

            <Allotment.Pane minSize={300}>
              <Allotment vertical>

                <Allotment.Pane minSize={120}>
                  {activeTab === 'tokens' && (
                    <Panel title="Tokens" count={outputs.lexer?.tokens?.length ?? 0} icon={Hash}>
                      <TokenPanel tokens={outputs.lexer?.tokens ?? []} />
                    </Panel>
                  )}
                  {activeTab === 'ir' && (
                    <Panel title="TAC / IR" count={outputs.ir?.instructions?.length ?? 0} icon={Code2}>
                      <TacList instrs={outputs.ir?.instructions ?? []} />
                    </Panel>
                  )}
                  {activeTab === 'ast' && (
                    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: 'var(--card)' }}>
                      <div style={{
                        display: 'flex', alignItems: 'center', gap: 8, padding: '0 12px', height: 34,
                        borderBottom: '1px solid var(--border)', background: 'var(--muted)', flexShrink: 0,
                      }}>
                        <GitBranch style={{ width: 12, height: 12, color: 'var(--muted-foreground)' }} />
                        <span style={{ fontSize: 10, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.08em', color: 'var(--muted-foreground)' }}>
                          AST
                        </span>
                        <span style={{ fontSize: 11, color: 'var(--muted-foreground)', marginLeft: 4 }}>
                          — click a node to jump to source
                        </span>
                      </div>
                      <div style={{ flex: 1, overflow: 'hidden' }}>
                        <AstView ast={outputs.rdAst?.ast ?? null} onNodeClick={highlightSpan} />
                      </div>
                    </div>
                  )}
                  {activeTab === 'reports' && (
                    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: 'var(--card)' }}>
                      <ReportTables ff={firstFollow} ll1={ll1Table} lr={lrTables} />
                    </div>
                  )}
                  {activeTab === 'vm' && (
                    <Panel title="VM Execution" icon={Terminal}>
                      <div style={{
                        display: 'flex', alignItems: 'center', gap: 8,
                        padding: '6px 12px', borderBottom: '1px solid var(--border)',
                      }}>
                        <label style={{ fontSize: 11, color: 'var(--muted-foreground)', whiteSpace: 'nowrap' }}>
                          Input:
                        </label>
                        <input
                          value={progInput}
                          onChange={e => setProgInput(e.target.value)}
                          placeholder="whitespace-separated values"
                          style={{
                            flex: 1, height: 28, borderRadius: 5,
                            border: '1px solid var(--border)',
                            background: 'var(--muted)', color: 'var(--foreground)',
                            fontSize: 12, fontFamily: 'monospace',
                            padding: '0 8px', outline: 'none',
                          }}
                        />
                      </div>
                      <VmOutputPanel out={outputs.vm} />
                    </Panel>
                  )}
                </Allotment.Pane>

                <Allotment.Pane minSize={80}>
                  <Allotment>
                    <Allotment.Pane>
                      <Panel title="Errors" count={errors.length} icon={AlertCircle}>
                        <ScrollArea className="h-full">
                          <ErrorPanel errors={errors} onJump={jumpTo} />
                        </ScrollArea>
                      </Panel>
                    </Allotment.Pane>
                    <Allotment.Pane>
                      <Panel title="Symbols" count={outputs.semantic?.symbol_snapshot?.length ?? 0} icon={Package}>
                        <ScrollArea className="h-full">
                          <SymbolTable entries={outputs.semantic?.symbol_snapshot ?? []} />
                        </ScrollArea>
                      </Panel>
                    </Allotment.Pane>
                  </Allotment>
                </Allotment.Pane>

              </Allotment>
            </Allotment.Pane>
          </Allotment>
        </div>

        {/* ── Pipeline stepper (demo mode) ──────────────────────────────── */}
        {stepMode && (
          <StepMode
            active={stepStage}
            onPrev={() => goStep(stepStage - 1)}
            onNext={() => goStep(stepStage + 1)}
          />
        )}

        {/* ── Status bar ────────────────────────────────────────────────── */}
        <div style={{
          display: 'flex', alignItems: 'center', gap: 12,
          height: 22, padding: '0 12px', flexShrink: 0,
          background: errors.length > 0 ? '#7c1a1a' : '#14532d',
          borderTop: '1px solid var(--border)', fontSize: 11, color: '#fff',
        }}>
          <span>{errors.length === 0 ? '✓ No errors' : `✗ ${errors.length} error${errors.length !== 1 ? 's' : ''}`}</span>
          <span style={{ marginLeft: 'auto', opacity: 0.7 }}>
            Pascal Subset Compiler · CS-471L
          </span>
        </div>

      </div>
    </TooltipProvider>
  );
}
