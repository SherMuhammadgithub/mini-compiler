import * as d3 from 'd3';
import { useEffect, useRef } from 'react';
import type { AstNode, Span } from '../types';

// ── Tree helpers ──────────────────────────────────────────────────────────────

function childrenOf(node: AstNode): AstNode[] {
  const k = node.kind;
  if (typeof k === 'string') return [];
  if ('Program' in k)                return [k.Program.declarations, k.Program.subprograms, k.Program.body];
  if ('Declarations' in k)           return k.Declarations.items;
  if ('VarDecl' in k)                return [k.VarDecl.ty];
  if ('TypeArray' in k)              return [k.TypeArray.element];
  if ('SubprogramDeclarations' in k) return k.SubprogramDeclarations.items;
  if ('FunctionDecl' in k)           return [...k.FunctionDecl.params, k.FunctionDecl.return_type, k.FunctionDecl.declarations, k.FunctionDecl.body];
  if ('ProcedureDecl' in k)          return [...k.ProcedureDecl.params, k.ProcedureDecl.declarations, k.ProcedureDecl.body];
  if ('ParamGroup' in k)             return [k.ParamGroup.ty];
  if ('CompoundStatement' in k)      return k.CompoundStatement.stmts;
  if ('Assignment' in k)             return [k.Assignment.target, k.Assignment.value];
  if ('ProcedureCall' in k)          return k.ProcedureCall.args;
  if ('IfStatement' in k)            return [k.IfStatement.cond, k.IfStatement.then_branch, ...(k.IfStatement.else_branch ? [k.IfStatement.else_branch] : [])];
  if ('WhileStatement' in k)         return [k.WhileStatement.cond, k.WhileStatement.body];
  if ('BinaryExpr' in k)             return [k.BinaryExpr.left, k.BinaryExpr.right];
  if ('UnaryExpr' in k)              return [k.UnaryExpr.operand];
  if ('Variable' in k)               return k.Variable.index ? [k.Variable.index] : [];
  if ('FunctionCall' in k)           return k.FunctionCall.args;
  return [];
}

function labelOf(node: AstNode): string {
  const k = node.kind;
  if (typeof k === 'string') return k;
  if ('Program' in k)                return `Program: ${k.Program.name}`;
  if ('Declarations' in k)           return 'Declarations';
  if ('VarDecl' in k)                return `VarDecl: ${k.VarDecl.names.join(', ')}`;
  if ('TypeArray' in k)              return `Array[${k.TypeArray.low}..${k.TypeArray.high}]`;
  if ('SubprogramDeclarations' in k) return 'Subprograms';
  if ('FunctionDecl' in k)           return `Fn: ${k.FunctionDecl.name}`;
  if ('ProcedureDecl' in k)          return `Proc: ${k.ProcedureDecl.name}`;
  if ('ParamGroup' in k)             return `Params: ${k.ParamGroup.names.join(', ')}`;
  if ('CompoundStatement' in k)      return 'Begin…End';
  if ('Assignment' in k)             return ':=';
  if ('ProcedureCall' in k)          return `Call: ${k.ProcedureCall.name}`;
  if ('IfStatement' in k)            return 'If';
  if ('WhileStatement' in k)         return 'While';
  if ('BinaryExpr' in k)             return k.BinaryExpr.op;
  if ('UnaryExpr' in k)              return k.UnaryExpr.op;
  if ('Variable' in k)               return `Var: ${k.Variable.name}`;
  if ('FunctionCall' in k)           return `Call: ${k.FunctionCall.name}`;
  if ('IntLiteral' in k)             return String(k.IntLiteral.value);
  if ('RealLiteral' in k)            return String(k.RealLiteral.value);
  return '?';
}

function nodeCategory(node: AstNode): string {
  const k = node.kind;
  if (typeof k === 'string') return 'leaf';
  if ('Program' in k) return 'program';
  if ('FunctionDecl' in k || 'ProcedureDecl' in k) return 'subprogram';
  if ('Declarations' in k || 'SubprogramDeclarations' in k ||
      'VarDecl' in k || 'TypeArray' in k || 'ParamGroup' in k) return 'decl';
  if ('CompoundStatement' in k || 'IfStatement' in k || 'WhileStatement' in k) return 'control';
  if ('Assignment' in k || 'ProcedureCall' in k) return 'stmt';
  if ('BinaryExpr' in k || 'UnaryExpr' in k) return 'expr';
  if ('Variable' in k || 'FunctionCall' in k ||
      'IntLiteral' in k || 'RealLiteral' in k) return 'value';
  return 'other';
}

// Chip colors: [fill, text] — dark solid chips matching token chip style
const CHIP: Record<string, [string, string]> = {
  program:    ['#1e3a8a', '#bfdbfe'],
  subprogram: ['#065f46', '#a7f3d0'],
  decl:       ['#14532d', '#bbf7d0'],
  control:    ['#7c2d12', '#fed7aa'],
  stmt:       ['#3f3f46', '#f4f4f5'],
  expr:       ['#78350f', '#fef3c7'],
  value:      ['#164e63', '#cffafe'],
  leaf:       ['#27272a', '#d4d4d8'],
  other:      ['#1c1917', '#f5f5f4'],
};

function countNodes(node: AstNode): number {
  return 1 + childrenOf(node).reduce((s, c) => s + countNodes(c), 0);
}

function textTree(node: AstNode, depth = 0): string {
  return [
    '  '.repeat(depth) + labelOf(node),
    ...childrenOf(node).map(c => textTree(c, depth + 1)),
  ].join('\n');
}

// ── Constants ─────────────────────────────────────────────────────────────────

const NODE_W = 110;
const NODE_H = 26;

// ── Component ─────────────────────────────────────────────────────────────────

export function AstView({ ast, onNodeClick }: {
  ast: AstNode | null;
  onNodeClick: (span: Span) => void;
}) {
  const svgRef = useRef<SVGSVGElement>(null);
  const cbRef  = useRef(onNodeClick);
  cbRef.current = onNodeClick;

  useEffect(() => {
    if (!svgRef.current) return;
    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();
    if (!ast) return;

    // text fallback for very large trees
    if (countNodes(ast) > 500) {
      const fo = svg.append('foreignObject').attr('width', '100%').attr('height', '100%');
      fo.append('xhtml:pre')
        .style('font', "12px/1.5 'Poppins', ui-sans-serif, sans-serif")
        .style('color', 'var(--color-foreground, #e4e4e7)')
        .style('padding', '8px')
        .style('margin', '0')
        .style('overflow', 'auto')
        .style('height', '100%')
        .text(textTree(ast));
      return;
    }

    // D3 tree layout
    const root = d3.hierarchy<AstNode>(ast, childrenOf);
    d3.tree<AstNode>().nodeSize([NODE_W + 20, 60])(root);

    const pts = root.descendants() as d3.HierarchyPointNode<AstNode>[];
    const lks = root.links()       as d3.HierarchyPointLink<AstNode>[];
    const cx  = (svgRef.current.clientWidth || 800) / 2;

    // pan + zoom
    const g    = svg.append('g');
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.08, 3])
      .on('zoom', e => g.attr('transform', e.transform.toString()));
    svg.call(zoom).call(zoom.transform, d3.zoomIdentity.translate(cx, 40));

    const linkColor = getComputedStyle(svgRef.current)
      .getPropertyValue('--color-border').trim() || '#333333';

    // connector lines
    g.selectAll<SVGPathElement, d3.HierarchyPointLink<AstNode>>('path')
      .data(lks).join('path')
      .attr('fill', 'none')
      .attr('stroke', linkColor)
      .attr('stroke-width', 1.5)
      .attr('d', ({ source: s, target: t }) =>
        `M${s.x},${s.y} C${s.x},${(s.y + t.y) / 2} ${t.x},${(s.y + t.y) / 2} ${t.x},${t.y}`
      );

    // chip nodes
    const node = g.selectAll<SVGGElement, d3.HierarchyPointNode<AstNode>>('g')
      .data(pts).join('g')
      .attr('transform', d => `translate(${d.x},${d.y})`)
      .style('cursor', 'pointer')
      .on('click', (_, d) => cbRef.current(d.data.span));

    node.append('title').text(d => labelOf(d.data));

    node.append('rect')
      .attr('x', -NODE_W / 2).attr('y', -NODE_H / 2)
      .attr('width', NODE_W).attr('height', NODE_H).attr('rx', 7)
      .attr('fill', d => CHIP[nodeCategory(d.data)]?.[0] ?? '#27272a')
      .attr('stroke', 'none');

    node.append('text')
      .attr('text-anchor', 'middle').attr('dominant-baseline', 'middle')
      .attr('fill', d => CHIP[nodeCategory(d.data)]?.[1] ?? '#e4e4e7')
      .attr('font-size', 11)
      .attr('font-weight', '500')
      .attr('font-family', "'Poppins', ui-sans-serif, system-ui, sans-serif")
      .text(d => { const l = labelOf(d.data); return l.length > 14 ? l.slice(0, 13) + '…' : l; });

  }, [ast]);

  return <svg ref={svgRef} width="100%" height="100%" />;
}
