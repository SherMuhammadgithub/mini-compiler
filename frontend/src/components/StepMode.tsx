import { ChevronLeft, ChevronRight, Check } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

const STAGES = [
  { label: 'Lexer',      sub: 'Tokenisation' },
  { label: 'RD Parser',  sub: 'Recursive Descent' },
  { label: 'LL(1)',      sub: 'Predictive' },
  { label: 'LR',         sub: 'Shift-Reduce' },
  { label: 'Symbols',    sub: 'Symbol Table' },
  { label: 'Semantic',   sub: 'Type Checking' },
  { label: 'IR / TAC',   sub: 'Three-Address Code' },
  { label: 'Code Gen',   sub: 'VM Bytecode' },
];

export function StepMode({
  active,
  onPrev,
  onNext,
}: {
  active: number;
  onPrev: () => void;
  onNext: () => void;
}) {
  return (
    <div className="flex items-center gap-3 px-4 py-2.5 bg-card border-t select-none">
      {/* Prev button */}
      <Button
        variant="outline"
        size="sm"
        onClick={onPrev}
        disabled={active === 0}
        className="h-7 w-7 p-0 shrink-0"
        aria-label="Previous stage"
      >
        <ChevronLeft className="h-4 w-4" />
      </Button>

      {/* Stepper */}
      <div className="flex flex-1 items-center min-w-0">
        {STAGES.map((stage, i) => {
          const isDone   = i < active;
          const isActive = i === active;
          const isPending = i > active;

          return (
            <div key={i} className="flex items-center flex-1 min-w-0">
              {/* Step node */}
              <div className="flex flex-col items-center shrink-0">
                {/* Circle */}
                <div
                  className={cn(
                    'flex items-center justify-center rounded-full text-xs font-bold transition-all',
                    isActive  && 'w-8 h-8 bg-primary text-primary-foreground shadow-md ring-4 ring-primary/20',
                    isDone    && 'w-6 h-6 bg-emerald-500 text-white',
                    isPending && 'w-6 h-6 bg-muted text-muted-foreground border border-border',
                  )}
                >
                  {isDone ? (
                    <Check className="h-3.5 w-3.5 stroke-3" />
                  ) : (
                    <span className={isActive ? 'text-xs' : 'text-[10px]'}>{i + 1}</span>
                  )}
                </div>

                {/* Label below circle */}
                <div className={cn(
                  'mt-1 text-center leading-tight transition-colors',
                  isActive  && 'text-primary',
                  isDone    && 'text-emerald-500',
                  isPending && 'text-muted-foreground',
                )}>
                  <div className={cn('font-semibold', isActive ? 'text-[11px]' : 'text-[10px]')}>
                    {stage.label}
                  </div>
                  <div className="text-[9px] opacity-70 hidden sm:block">{stage.sub}</div>
                </div>
              </div>

              {/* Connector line (not after last) */}
              {i < STAGES.length - 1 && (
                <div className={cn(
                  'flex-1 h-px mx-1 transition-colors',
                  i < active ? 'bg-emerald-500' : 'bg-border',
                )} />
              )}
            </div>
          );
        })}
      </div>

      {/* Next button */}
      <Button
        variant="outline"
        size="sm"
        onClick={onNext}
        disabled={active === STAGES.length - 1}
        className="h-7 w-7 p-0 shrink-0"
        aria-label="Next stage"
      >
        <ChevronRight className="h-4 w-4" />
      </Button>
    </div>
  );
}
