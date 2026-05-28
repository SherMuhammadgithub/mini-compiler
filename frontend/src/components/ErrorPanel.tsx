import { AlertCircle, AlertTriangle, CheckCircle2 } from 'lucide-react';
import type { CompilerError } from '../types';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';

const STAGE_COLOR: Record<string, string> = {
  lexer:      'bg-amber-500/15 text-amber-500 border-amber-500/30',
  rd_parser:  'bg-blue-500/15 text-blue-500 border-blue-500/30',
  ll1_parser: 'bg-violet-500/15 text-violet-500 border-violet-500/30',
  lr_parser:  'bg-purple-500/15 text-purple-500 border-purple-500/30',
  semantic:   'bg-rose-500/15 text-rose-500 border-rose-500/30',
  symbol_table: 'bg-orange-500/15 text-orange-500 border-orange-500/30',
};

export function ErrorPanel({
  errors,
  onJump,
}: {
  errors: CompilerError[];
  onJump: (line: number, col: number) => void;
}) {
  if (errors.length === 0) {
    return (
      <div className="flex items-center gap-2 px-3 py-2.5 text-sm text-emerald-500">
        <CheckCircle2 className="h-4 w-4 shrink-0" />
        <span className="font-medium">No errors</span>
      </div>
    );
  }

  return (
    <div className="divide-y divide-border">
      {errors.map((e, i) => (
        <button
          key={i}
          onClick={() => onJump(e.line, e.column)}
          className={cn(
            'w-full flex items-start gap-2.5 px-3 py-2 text-left text-xs transition-colors hover:bg-muted/60',
          )}
        >
          {e.severity === 'error'
            ? <AlertCircle className="mt-px h-3.5 w-3.5 shrink-0 text-destructive" />
            : <AlertTriangle className="mt-px h-3.5 w-3.5 shrink-0 text-amber-500" />
          }
          <div className="flex flex-col gap-0.5 min-w-0">
            <div className="flex items-center gap-1.5 flex-wrap">
              <Badge
                className={cn('text-[10px] h-4 px-1.5 border', STAGE_COLOR[e.stage] ?? 'bg-muted text-muted-foreground')}
                variant="outline"
              >
                {e.stage}
              </Badge>
              <span className="font-mono text-muted-foreground">L{e.line}:{e.column}</span>
            </div>
            <span className="text-foreground leading-snug">{e.message}</span>
          </div>
        </button>
      ))}
    </div>
  );
}
