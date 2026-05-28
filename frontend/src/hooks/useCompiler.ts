import { useState, useEffect, useRef } from 'react';
import { loadWasm } from './useWasm';
import type {
  LexerOutput, RdParseOutput, Ll1ParseOutput, LrParseOutput,
  SemanticOutput, IrOutput, CodegenOutput, VmOutput,
} from '../types';

export interface CompilerOutputs {
  lexer:    LexerOutput   | null;
  rdAst:    RdParseOutput | null;
  ll1:      Ll1ParseOutput | null;
  lr:       LrParseOutput | null;
  semantic: SemanticOutput | null;
  ir:       IrOutput       | null;
  codegen:  CodegenOutput  | null;
  vm:       VmOutput       | null;
  loading:  boolean;
}

const EMPTY: CompilerOutputs = {
  lexer: null, rdAst: null, ll1: null, lr: null,
  semantic: null, ir: null, codegen: null, vm: null, loading: false,
};

export function useCompiler(source: string, programInput: string): CompilerOutputs {
  const [outputs, setOutputs] = useState<CompilerOutputs>(EMPTY);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    clearTimeout(timer.current);
    setOutputs(prev => ({ ...prev, loading: true }));

    timer.current = setTimeout(async () => {
      try {
        const wasm = await loadWasm();
        setOutputs({
          lexer:    JSON.parse(wasm.run_lexer(source)),
          rdAst:    JSON.parse(wasm.run_rd_parser(source)),
          ll1:      JSON.parse(wasm.run_ll1_parser(source)),
          lr:       JSON.parse(wasm.run_lr_parser(source)),
          semantic: JSON.parse(wasm.run_symbol_table(source)),
          ir:       JSON.parse(wasm.run_ir(source)),
          codegen:  JSON.parse(wasm.run_codegen(source)),
          vm:       JSON.parse(wasm.run_program(source, programInput)),
          loading:  false,
        });
      } catch (err) {
        console.error('WASM pipeline error', err);
        setOutputs(prev => ({ ...prev, loading: false }));
      }
    }, 150);

    return () => clearTimeout(timer.current);
  }, [source, programInput]);

  return outputs;
}
