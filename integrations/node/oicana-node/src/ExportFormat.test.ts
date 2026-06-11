import { describe, expect, it } from 'vitest';
import { Png } from './ExportFormat';

describe('Png', () => {
  it('accepts positive, finite resolutions', () => {
    expect(Png(0.5)).toEqual({ format: 'png', pixelsPerPt: 0.5 });
    expect(Png(2)).toEqual({ format: 'png', pixelsPerPt: 2 });
  });

  it('rejects degenerate resolutions', () => {
    for (const pixelsPerPt of [0, -1, Number.NaN, Number.POSITIVE_INFINITY]) {
      expect(() => Png(pixelsPerPt)).toThrow(TypeError);
    }
  });

  it('rejects non-numbers passed from untyped callers', () => {
    expect(() => Png(new Map() as unknown as number)).toThrow(
      /pixelsPerPt must be a positive, finite number/,
    );
  });
});
