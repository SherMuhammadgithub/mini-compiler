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
        .style('font', '12px/1.5 ui-monospace, Consolas, monospace')
        .style('color', '#C9D1D9')
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

    // links (cubic bezier)
    g.selectAll<SVGPathElement, d3.HierarchyPointLink<AstNode>>('path')
      .data(lks).join('path')
      .attr('fill', 'none')
      .attr('stroke', '#30363D')
      .attr('stroke-width', 1.5)
      .attr('d', ({ source: s, target: t }) =>
        `M${s.x},${s.y} C${s.x},${(s.y + t.y) / 2} ${t.x},${(s.y + t.y) / 2} ${t.x},${t.y}`
      );

    // nodes
    const node = g.selectAll<SVGGElement, d3.HierarchyPointNode<AstNode>>('g')
      .data(pts).join('g')
      .attr('transform', d => `translate(${d.x},${d.y})`)
      .style('cursor', 'pointer')
      .on('click', (_, d) => cbRef.current(d.data.span));

    node.append('title').text(d => labelOf(d.data));

    node.append('rect')
      .attr('x', -NODE_W / 2).attr('y', -NODE_H / 2)
      .attr('width', NODE_W).attr('height', NODE_H).attr('rx', 5)
      .attr('fill', '#1E2530').attr('stroke', '#7C3AED').attr('stroke-width', 1.5);

    node.append('text')
      .attr('text-anchor', 'middle').attr('dominant-baseline', 'middle')
      .attr('fill', '#C9D1D9').attr('font-size', 11)
      .attr('font-family', 'ui-monospace, Consolas, monospace')
      .text(d => { const l = labelOf(d.data); return l.length > 14 ? l.slice(0, 13) + '…' : l; });

  }, [ast]);

  return <svg ref={svgRef} width="100%" height="100%" />;
}
