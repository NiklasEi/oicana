export type ExportFormat =
  | { format: 'pdf' | 'svg' }
  | { format: 'png'; pixelsPerPt: number };

export const Pdf: ExportFormat = { format: 'pdf' };
export const Svg: ExportFormat = { format: 'svg' };
export const Png = (pixelsPerPt: number): ExportFormat => {
  if (
    typeof pixelsPerPt !== 'number' ||
    !Number.isFinite(pixelsPerPt) ||
    pixelsPerPt <= 0
  ) {
    throw new TypeError(
      `pixelsPerPt must be a positive, finite number, got ${String(pixelsPerPt)}`,
    );
  }
  return { format: 'png', pixelsPerPt };
};
