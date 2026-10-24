import { vi } from 'vitest'

/** jsdom has no layout: zero-sized handles otherwise overlap every synthetic click. */
export function mockPanelGeometry() {
  const original = HTMLElement.prototype.getBoundingClientRect
  return vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (
    this: HTMLElement,
  ) {
    if (this.hasAttribute('data-panel-resize-handle-id')) return new DOMRect(500, 0, 1, 800)
    if (this.hasAttribute('data-panel-group-id')) return new DOMRect(0, 0, 1000, 800)
    return original.call(this)
  })
}
