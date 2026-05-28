import type { SymbolEntry, PascalType } from '../types';
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

const SCOPE_COLORS = [
  'bg-blue-500/10 text-blue-500 border-blue-500/30',
  'bg-emerald-500/10 text-emerald-500 border-emerald-500/30',
  'bg-amber-500/10 text-amber-500 border-amber-500/30',
  'bg-violet-500/10 text-violet-500 border-violet-500/30',
];

const KIND_COLORS: Record<string, string> = {
  variable:  'bg-blue-500/10 text-blue-500 border-blue-500/30',
  function:  'bg-violet-500/10 text-violet-500 border-violet-500/30',
  procedure: 'bg-teal-500/10 text-teal-500 border-teal-500/30',
  parameter: 'bg-amber-500/10 text-amber-500 border-amber-500/30',
  program:   'bg-muted text-muted-foreground border-border',
};

function typeLabel(t: PascalType): string {
  if (typeof t === 'string') return t;
  if ('Array' in t)
    return `array[${t.Array.low}..${t.Array.high}] of ${typeLabel(t.Array.element)}`;
  return JSON.stringify(t);
}

export function SymbolTable({ entries }: { entries: SymbolEntry[] }) {
  if (entries.length === 0) {
    return (
      <div className="flex items-center justify-center h-16 text-sm text-muted-foreground italic">
        No symbols
      </div>
    );
  }

  return (
    <Table>
      <TableHeader>
        <TableRow className="border-b-2">
          <TableHead>Name</TableHead>
          <TableHead>Kind</TableHead>
          <TableHead>Type</TableHead>
          <TableHead className="text-center">Scope</TableHead>
          <TableHead className="text-center">Line</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {entries.map((e, i) => (
          <TableRow key={i}>
            <TableCell>
              <span
                className="font-mono font-medium text-foreground"
                style={{ paddingLeft: e.scope_level * 12 }}
              >
                {e.scope_level > 0 && (
                  <span className="text-muted-foreground mr-1">{'└'}</span>
                )}
                {e.name}
              </span>
            </TableCell>
            <TableCell>
              <Badge
                variant="outline"
                className={cn('text-[10px] h-4 px-1.5 border', KIND_COLORS[String(e.kind)] ?? 'bg-muted text-muted-foreground')}
              >
                {String(e.kind)}
              </Badge>
            </TableCell>
            <TableCell>
              <span className="font-mono text-xs text-muted-foreground">{typeLabel(e.pascal_type)}</span>
            </TableCell>
            <TableCell className="text-center">
              <Badge
                variant="outline"
                className={cn('text-[10px] h-4 w-6 justify-center border', SCOPE_COLORS[e.scope_level % SCOPE_COLORS.length])}
              >
                {e.scope_level}
              </Badge>
            </TableCell>
            <TableCell className="text-center font-mono text-xs text-muted-foreground">
              {e.line}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
