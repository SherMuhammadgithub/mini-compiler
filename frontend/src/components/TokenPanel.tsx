import type { Token, TokenKind } from '../types';
import { cn } from '@/lib/utils';

const KEYWORDS = new Set([
  'Program','Var','Array','Of','Integer','Real','Function','Procedure',
  'Begin','End','If','Then','Else','While','Do','Not','And','Or','Div','Mod',
]);

export function categoryOf(kind: TokenKind): string {
  if (typeof kind === 'string') {
    if (KEYWORDS.has(kind))  return 'keyword';
    if (kind === 'Id')       return 'identifier';
    if (kind === 'Num')      return 'number';
    if (kind === 'Assignop') return 'operator';
    if (kind === 'Unknown')  return 'error';
    if (['LParen','RParen','LBracket','RBracket',
         'Semicolon','Colon','Comma','Dot','DotDot'].includes(kind)) return 'punctuation';
  }
  if (typeof kind === 'object' &&
      ('Relop' in kind || 'Addop' in kind || 'Mulop' in kind)) return 'operator';
  return 'punctuation';
}

export function kindLabel(kind: TokenKind): string {
  if (typeof kind === 'string') return kind;
  if ('Relop' in kind) return `Relop`;
  if ('Addop' in kind) return `Addop`;
  if ('Mulop' in kind) return `Mulop`;
  return '?';
}

const CHIP_STYLES: Record<string, string> = {
  keyword:     'bg-violet-700 text-white border-violet-600',
  identifier:  'bg-blue-600 text-white border-blue-500',
  number:      'bg-emerald-700 text-white border-emerald-600',
  operator:    'bg-amber-600 text-white border-amber-500',
  punctuation: 'bg-zinc-600 text-zinc-100 border-zinc-500',
  error:       'bg-red-700 text-white border-red-600',
};

export function TokenPanel({ tokens }: { tokens: Token[] }) {
  const filtered = tokens.filter(t => t.kind !== 'Eof');
  if (filtered.length === 0) {
    return (
      <div className="flex items-center justify-center h-20 text-sm text-muted-foreground italic">
        No tokens yet
      </div>
    );
  }

  return (
    <div className="flex flex-wrap gap-2 p-3">
      {filtered.map((tok, i) => {
        const cat = categoryOf(tok.kind);
        return (
          <span
            key={i}
            title={`${kindLabel(tok.kind)} · Line ${tok.line}, Col ${tok.column}`}
            className={cn(
              'inline-flex items-center gap-1 rounded-md border px-2.5 py-1 font-mono leading-tight',
              CHIP_STYLES[cat],
            )}
          >
            <span style={{ fontSize: 10, opacity: 0.7 }}>{kindLabel(tok.kind)}</span>
            <span style={{ fontSize: 12, fontWeight: 700 }}>{tok.lexeme}</span>
          </span>
        );
      })}
    </div>
  );
}
