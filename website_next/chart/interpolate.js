/**
 * Interpolate nonempty points ordered by increasing plotX (ties allowed).
 * @template {{ plotX: number }} T
 * @param {T[]} points
 * @param {number} plotX
 * @param {(point: T) => number} read
 */
export function interpolatePlotValue(points, plotX, read) {
  if (plotX <= points[0].plotX) return read(points[0]);

  let lower = 1;
  let upper = points.length;
  while (lower < upper) {
    const middle = Math.floor((lower + upper) / 2);
    if (plotX > points[middle].plotX) lower = middle + 1;
    else upper = middle;
  }

  if (lower === points.length) return read(points[points.length - 1]);

  const previous = points[lower - 1];
  const next = points[lower];
  const span = next.plotX - previous.plotX;
  const ratio = span ? (plotX - previous.plotX) / span : 0;

  return read(previous) + (read(next) - read(previous)) * ratio;
}
