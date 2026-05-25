import type { Token, TokenKind } from '../types';

const KIND_COLOR: Record<string, string> = {
  keyword:     '#7C3AED',
  identifier:  '#2563EB',
  number:      '#0891B2',
  operator:    '#D97706',
  punctuation: '#6B7280',
  error:       '#DC2626',
};

const KEYWORDS = new Set([
  'Program','Var','Array','Of','Integer','Real','Function','Procedure',
  'Begin','End','If','Then','Else','While','Do','Not','And','Or','Div','Mod',
]);

export function categoryOf(kind: TokenKind): string {
  if (typeof kind === 'string') {
    if (KEYWORDS.has(kind))       return 'keyword';
    if (kind === 'Id')            return 'identifier';
    if (kind === 'Num')           return 'number';
    if (kind === 'Assignop')      return 'operator';
    if (kind === 'Unknown')       return 'error';
    if (['LParen','RParen','LBracket','RBracket',
         'Semicolon','Colon','Comma','Dot','DotDot'].includes(kind)) return 'punctuation';
  }
  if (typeof kind === 'object' &&
      ('Relop' in kind || 'Addop' in kind || 'Mulop' in kind)) return 'operator';
  return 'punctuation';
}

export function kindLabel(kind: TokenKind): string {
  if (typeof kind === 'string') return kind;
  if ('Relop' in kind) return `Relop(${(kind as { Relop: string }).Relop})`;
  if ('Addop' in kind) return `Addop(${(kind as { Addop: string }).Addop})`;
  if ('Mulop' in kind) return `Mulop(${(kind as { Mulop: string }).Mulop})`;
  return '?';
}

export function TokenPanel({ tokens }: { tokens: Token[] }) {
  return (
    <div className="chip-wrap">
      {tokens.filter(t => t.kind !== 'Eof').map((tok, i) => (
        <span
          key={i}
          className="chip"
          style={{ background: KIND_COLOR[categoryOf(tok.kind)] }}
          title={`Line ${tok.line}, Col ${tok.column}`}
        >
          {kindLabel(tok.kind)}: {tok.lexeme}
        </span>
      ))}
    </div>
  );
}
