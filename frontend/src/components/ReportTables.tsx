import { Printer } from 'lucide-react';
import type { FirstFollowOutput, Ll1TableOutput, LrTableOutput } from '../types';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/table';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { cn } from '@/lib/utils';

function EmptyState({ label }: { label: string }) {
  return (
    <div className="flex items-center justify-center h-24 text-sm text-muted-foreground italic">
      {label}
    </div>
  );
}

function FirstFollowTable({ ff }: { ff: FirstFollowOutput }) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-40">Non-Terminal</TableHead>
          <TableHead>FIRST</TableHead>
          <TableHead>FOLLOW</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {ff.rows.map((row, i) => (
          <TableRow key={i}>
            <TableCell className="font-mono font-semibold text-primary">{row.non_terminal}</TableCell>
            <TableCell className="font-mono text-xs text-muted-foreground">
              {'{' + row.first.join(', ') + '}'}
            </TableCell>
            <TableCell className="font-mono text-xs text-muted-foreground">
              {'{' + row.follow.join(', ') + '}'}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}

function Ll1TableView({ ll1 }: { ll1: Ll1TableOutput }) {
  return (
    <div className="overflow-x-auto">
      <table className="text-xs border-collapse min-w-full">
        <thead>
          <tr>
            <th className="sticky left-0 z-10 bg-muted px-3 py-2 text-left font-semibold text-muted-foreground border-b border-r min-w-32">
              NT \ Terminal
            </th>
            {ll1.terminals.map(t => (
              <th key={t} className="px-2 py-2 text-center font-mono font-medium text-muted-foreground border-b border-r whitespace-nowrap">
                {t}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {ll1.rows.map((row, i) => (
            <tr key={i} className="hover:bg-muted/40 transition-colors">
              <td className="sticky left-0 z-10 bg-card px-3 py-1.5 font-mono font-semibold text-primary border-b border-r">
                {row.non_terminal}
              </td>
              {ll1.terminals.map(t => (
                <td
                  key={t}
                  className={cn(
                    'px-2 py-1.5 text-center font-mono border-b border-r',
                    row.cells[t] ? 'text-foreground' : 'text-muted-foreground/30',
                  )}
                >
                  {row.cells[t] ?? '—'}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function LrActionTable({ lr }: { lr: LrTableOutput }) {
  return (
    <div className="overflow-x-auto">
      <table className="text-xs border-collapse min-w-full">
        <thead>
          <tr>
            <th className="sticky left-0 z-10 bg-muted px-3 py-2 text-left font-semibold text-muted-foreground border-b border-r">
              State
            </th>
            {lr.terminals.map(t => (
              <th key={t} className="px-2 py-2 text-center font-mono font-medium text-muted-foreground border-b border-r whitespace-nowrap">
                {t}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {lr.action_rows.map((row, i) => (
            <tr key={i} className="hover:bg-muted/40 transition-colors">
              <td className="sticky left-0 z-10 bg-card px-3 py-1.5 font-mono font-semibold text-muted-foreground border-b border-r">
                {i}
              </td>
              {lr.terminals.map(t => (
                <td
                  key={t}
                  className={cn(
                    'px-2 py-1.5 text-center font-mono border-b border-r',
                    row[t] === 'acc'             ? 'text-emerald-500 font-bold' :
                    row[t]?.startsWith('s')      ? 'text-blue-400' :
                    row[t]?.startsWith('r')      ? 'text-amber-400' :
                    'text-muted-foreground/30',
                  )}
                >
                  {row[t] ?? ''}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function LrGotoTable({ lr }: { lr: LrTableOutput }) {
  return (
    <div className="overflow-x-auto">
      <table className="text-xs border-collapse min-w-full">
        <thead>
          <tr>
            <th className="sticky left-0 z-10 bg-muted px-3 py-2 text-left font-semibold text-muted-foreground border-b border-r">
              State
            </th>
            {lr.non_terminals.map(n => (
              <th key={n} className="px-2 py-2 text-center font-mono font-medium text-muted-foreground border-b border-r whitespace-nowrap">
                {n}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {lr.goto_rows.map((row, i) => (
            <tr key={i} className="hover:bg-muted/40 transition-colors">
              <td className="sticky left-0 z-10 bg-card px-3 py-1.5 font-mono font-semibold text-muted-foreground border-b border-r">
                {i}
              </td>
              {lr.non_terminals.map(n => (
                <td
                  key={n}
                  className={cn(
                    'px-2 py-1.5 text-center font-mono border-b border-r',
                    row[n] != null ? 'text-sky-400' : 'text-muted-foreground/30',
                  )}
                >
                  {row[n] ?? ''}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function ReportTables({ ff, ll1, lr }: {
  ff:  FirstFollowOutput | null;
  ll1: Ll1TableOutput    | null;
  lr:  LrTableOutput     | null;
}) {
  return (
    <div className="flex flex-col h-full">
      <Tabs defaultValue="ff" className="flex flex-col flex-1 min-h-0">
        <div className="flex items-center justify-between px-3 py-1.5 border-b shrink-0">
          <TabsList>
            <TabsTrigger value="ff">FIRST / FOLLOW</TabsTrigger>
            <TabsTrigger value="ll1">LL(1) Table</TabsTrigger>
            <TabsTrigger value="action">LR Action</TabsTrigger>
            <TabsTrigger value="goto">LR Goto</TabsTrigger>
          </TabsList>
          <Button variant="ghost" size="sm" onClick={() => window.print()} className="h-7 gap-1.5 text-xs">
            <Printer className="h-3.5 w-3.5" />
            Print
          </Button>
        </div>

        <TabsContent value="ff" className="flex-1 min-h-0">
          <ScrollArea className="h-full">
            {ff ? <FirstFollowTable ff={ff} /> : <EmptyState label="Loading FIRST/FOLLOW…" />}
          </ScrollArea>
        </TabsContent>

        <TabsContent value="ll1" className="flex-1 min-h-0">
          <div className="overflow-auto h-full">
            <div className="p-2">
              {ll1 ? <Ll1TableView ll1={ll1} /> : <EmptyState label="Loading LL(1) table…" />}
            </div>
          </div>
        </TabsContent>

        <TabsContent value="action" className="flex-1 min-h-0">
          <div className="overflow-auto h-full">
            <div className="p-2">
              {lr ? <LrActionTable lr={lr} /> : <EmptyState label="Loading LR action table…" />}
            </div>
          </div>
        </TabsContent>

        <TabsContent value="goto" className="flex-1 min-h-0">
          <div className="overflow-auto h-full">
            <div className="p-2">
              {lr ? <LrGotoTable lr={lr} /> : <EmptyState label="Loading LR goto table…" />}
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
