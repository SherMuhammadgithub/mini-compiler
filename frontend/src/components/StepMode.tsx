const STAGES = [
  'Lexer', 'RD Parser', 'LL(1) Parser', 'LR Parser',
  'Symbol Table', 'Semantic', 'IR / TAC', 'Code Gen',
];

export function StepMode({ active, onPrev, onNext }: {
  active: number;
  onPrev: () => void;
  onNext: () => void;
}) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
      <button
        onClick={onPrev}
        disabled={active === 0}
        style={{ padding: '4px 10px', cursor: active === 0 ? 'default' : 'pointer' }}
      >
        ◀ Prev
      </button>
      <span style={{ fontWeight: 600, minWidth: 120, textAlign: 'center', color: 'var(--text)' }}>
        {STAGES[active]}
      </span>
      <button
        onClick={onNext}
        disabled={active === STAGES.length - 1}
        style={{ padding: '4px 10px', cursor: active === STAGES.length - 1 ? 'default' : 'pointer' }}
      >
        Next ▶
      </button>
      <div style={{ display: 'flex', gap: 4, marginLeft: 4 }}>
        {STAGES.map((s, i) => (
          <div
            key={i}
            title={s}
            style={{
              width: 10, height: 10, borderRadius: '50%',
              background: i === active ? '#2563EB' : i < active ? '#7C3AED' : '#D1D5DB',
            }}
          />
        ))}
      </div>
    </div>
  );
}
